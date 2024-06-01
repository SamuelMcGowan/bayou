use std::ops::ControlFlow::{self, Break, Continue};

use bayou_ir::ir::{Block as IrBlock, *};
use bayou_ir::symbols::LocalId;
use bayou_ir::{BinOp, Type as IrType, UnOp};
use bayou_session::diagnostics::DiagnosticEmitter;
use bayou_session::Session;
use cranelift::codegen::verify_function;
use cranelift::prelude::*;
use cranelift_module::{Linkage, Module as _};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use target_lexicon::Triple;

use crate::layout::{ConstantAsImm, TypeExt, TypeLayout};
use crate::{BackendError, BackendResult};

struct UnreachableCode;

pub struct Codegen<'sess, D: DiagnosticEmitter> {
    session: &'sess mut Session<D>,

    ctx: codegen::Context,
    builder_ctx: FunctionBuilderContext,
    module: ObjectModule,
}

impl<'sess, D: DiagnosticEmitter> Codegen<'sess, D> {
    pub fn new(
        session: &'sess mut Session<D>,
        target: Triple,
        package_name: &str,
    ) -> BackendResult<Self> {
        let mut flag_builder = settings::builder();
        flag_builder.set("is_pic", "true").unwrap();
        flag_builder.set("opt_level", "speed").unwrap();

        let flags = settings::Flags::new(flag_builder);

        let isa = match isa::lookup(target.clone()) {
            Ok(isa_builder) => isa_builder.finish(flags)?,
            Err(_) => {
                return Err(BackendError::UnsupportedArch(target.architecture));
            }
        };

        let module_builder =
            ObjectBuilder::new(isa, package_name, cranelift_module::default_libcall_names())?;

        let module = ObjectModule::new(module_builder);

        Ok(Self {
            session,

            ctx: module.make_context(),
            builder_ctx: FunctionBuilderContext::new(),
            module,
        })
    }

    pub fn compile_module(
        &mut self,
        module: &Module,
        module_cx: &ModuleContext,
    ) -> BackendResult<()> {
        for item in &module.items {
            match item {
                Item::FuncDecl(func_decl) => self.gen_func_decl(func_decl, module_cx)?,
            }
        }

        Ok(())
    }

    pub fn finish(self) -> BackendResult<ObjectProduct> {
        Ok(self.module.finish())
    }

    fn gen_func_decl(
        &mut self,
        func_decl: &FuncDecl,
        module_cx: &ModuleContext,
    ) -> BackendResult<()> {
        self.module.clear_context(&mut self.ctx);

        let func_symbol = &module_cx.symbols.funcs[func_decl.id];

        match func_symbol.ret_ty.layout() {
            TypeLayout::Integer(ty) => {
                self.ctx.func.signature.returns.push(AbiParam::new(ty));
            }
            TypeLayout::Void | TypeLayout::Never => {}
        }

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block); // no predecessors

        // function codegen
        let mut func_codegen = FuncCodegen { builder, module_cx };

        if let ControlFlow::Continue(val) = func_codegen.gen_block_expr(&func_decl.block) {
            match val {
                RValue::Value(val, _) => {
                    func_codegen.builder.ins().return_(&[val]);
                }
                RValue::Void => {
                    func_codegen.builder.ins().return_(&[]);
                }
            }
        }

        func_codegen.builder.finalize();

        // Any error returned here is a compiler bug.
        // TODO: should there be a feature flag for stuff like this?
        verify_function(&self.ctx.func, self.module.isa()).expect("function verification failed");

        // declare and define in module (not final)
        let name = self.session.interner.get(func_symbol.ident.ident_str);
        let id = self
            .module
            .declare_function(name, Linkage::Export, &self.ctx.func.signature)?;
        self.module.define_function(id, &mut self.ctx)?;

        Ok(())
    }
}

enum RValue {
    Value(Value, Type),
    Void,
}

struct FuncCodegen<'a> {
    builder: FunctionBuilder<'a>,
    module_cx: &'a ModuleContext,
}

impl FuncCodegen<'_> {
    fn gen_stmt(&mut self, stmt: &Stmt) -> ControlFlow<UnreachableCode> {
        match stmt {
            Stmt::Assign { local, expr } => self.gen_assignment_stmt(*local, expr),
            Stmt::Drop(expr) => {
                // just ignore the generated value
                let _ = self.gen_expr(expr)?;
                Continue(())
            }
            Stmt::Return(expr) => self.gen_return_stmt(expr),
        }
    }

    fn gen_assignment_stmt(&mut self, local: LocalId, expr: &Expr) -> ControlFlow<UnreachableCode> {
        let var = Variable::new(local.0);

        match self.gen_expr(expr)? {
            RValue::Value(val, ty) => {
                self.builder.declare_var(var, ty);
                self.builder.def_var(var, val);
            }
            RValue::Void => {}
        }

        Continue(())
    }

    fn gen_return_stmt(&mut self, expr: &Expr) -> ControlFlow<UnreachableCode> {
        match self.gen_expr(expr)? {
            RValue::Value(val, _ty) => {
                self.builder.ins().return_(&[val]);
            }
            RValue::Void => {}
        }

        let after_return = self.builder.create_block();
        self.builder.switch_to_block(after_return);
        self.builder.seal_block(after_return); // nothing jumps here, dead code

        // Anything after a return is unreachable
        Break(UnreachableCode)
    }

    fn gen_expr(&mut self, expr: &Expr) -> ControlFlow<UnreachableCode, RValue> {
        match &expr.kind {
            ExprKind::Constant(constant) => Continue(self.gen_constant_expr(constant)),
            ExprKind::Var(local) => Continue(self.gen_var_expr(*local)),
            ExprKind::UnOp { op, expr } => self.gen_unop_expr(*op, expr),
            ExprKind::BinOp { op, lhs, rhs } => self.gen_binop_expr(*op, lhs, rhs),
            ExprKind::Block(block) => self.gen_block_expr(block),
            ExprKind::If { cond, then, else_ } => {
                self.gen_if_expr(cond, then, else_.as_deref(), expr.ty.unwrap())
            }
        }
    }

    fn gen_constant_expr(&mut self, constant: &Constant) -> RValue {
        match constant.ty().layout() {
            TypeLayout::Integer(ty) => {
                // constant must have an immediate because it is an integer
                let val = self.builder.ins().iconst(ty, constant.as_imm().unwrap());
                RValue::Value(val, ty)
            }
            TypeLayout::Void => RValue::Void,
            TypeLayout::Never => unreachable!(),
        }
    }

    fn gen_var_expr(&mut self, local: LocalId) -> RValue {
        // variables of type never and variables with expressions containing unreachable code will have stopped the codegen by now, so
        // we don't need to worry about them

        let local_ty = self.module_cx.symbols.locals[local].ty;
        let layout = local_ty.layout();

        match layout {
            TypeLayout::Integer(ty) => {
                let var = Variable::new(local.0);
                RValue::Value(self.builder.use_var(var), ty)
            }
            TypeLayout::Void => RValue::Void,
            TypeLayout::Never => unreachable!(),
        }
    }

    fn gen_unop_expr(&mut self, op: UnOp, expr: &Expr) -> ControlFlow<UnreachableCode, RValue> {
        let expr = match self.gen_expr(expr)? {
            RValue::Value(value, _) => value,
            RValue::Void => unreachable!(),
        };

        let val = match op {
            UnOp::Negate => self.builder.ins().ineg(expr),
            UnOp::BitwiseInvert => self.builder.ins().bnot(expr),
        };

        Continue(RValue::Value(val, types::I64))
    }

    fn gen_binop_expr(
        &mut self,
        op: BinOp,
        lhs: &Expr,
        rhs: &Expr,
    ) -> ControlFlow<UnreachableCode, RValue> {
        let lhs = match self.gen_expr(lhs)? {
            RValue::Value(value, _) => value,
            RValue::Void => unreachable!(),
        };

        let rhs = match self.gen_expr(rhs)? {
            RValue::Value(value, _) => value,
            RValue::Void => unreachable!(),
        };

        let ins = self.builder.ins();
        let val = match op {
            BinOp::Add => ins.iadd(lhs, rhs),
            BinOp::Sub => ins.isub(lhs, rhs),
            BinOp::Mul => ins.imul(lhs, rhs),
            BinOp::Div => ins.sdiv(lhs, rhs),
            BinOp::Mod => ins.srem(lhs, rhs),
            BinOp::BitwiseAnd => ins.band(lhs, rhs),
            BinOp::BitwiseOr => ins.bor(lhs, rhs),
            BinOp::BitwiseXor => ins.bxor(lhs, rhs),
        };

        Continue(RValue::Value(val, types::I64))
    }

    fn gen_block_expr(&mut self, block: &IrBlock) -> ControlFlow<UnreachableCode, RValue> {
        for stmt in &block.statements {
            self.gen_stmt(stmt)?;
        }
        self.gen_expr(&block.final_expr)
    }

    fn gen_if_expr(
        &mut self,
        cond: &Expr,
        then: &Expr,
        else_: Option<&Expr>,
        ty: IrType,
    ) -> ControlFlow<UnreachableCode, RValue> {
        let cond = match self.gen_expr(cond)? {
            RValue::Value(value, _) => value,
            RValue::Void => unreachable!(),
        };

        let next_block = self.builder.create_block();
        match ty.layout() {
            TypeLayout::Integer(ty) => {
                self.builder.append_block_param(next_block, ty);
            }
            TypeLayout::Void | TypeLayout::Never => {}
        }

        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();

        self.builder
            .ins()
            .brif(cond, then_block, &[], else_block, &[]);

        // only the previous branch can jump to these blocks
        self.builder.seal_block(then_block);
        self.builder.seal_block(else_block);

        // then branch
        self.builder.switch_to_block(then_block);
        match self.gen_expr(then)? {
            RValue::Value(val, _) => {
                self.builder.ins().jump(next_block, &[val]);
            }
            RValue::Void => {
                self.builder.ins().jump(next_block, &[]);
            }
        }

        // else branch
        self.builder.switch_to_block(else_block);
        if let Some(else_) = else_ {
            match self.gen_expr(else_)? {
                RValue::Value(val, _) => {
                    self.builder.ins().jump(next_block, &[val]);
                }
                RValue::Void => {
                    self.builder.ins().jump(next_block, &[]);
                }
            }
        } else {
            self.builder.ins().jump(next_block, &[]);
        }

        // next block
        self.builder.seal_block(next_block);
        self.builder.switch_to_block(next_block);

        let if_value = match ty.layout() {
            TypeLayout::Integer(ty) => RValue::Value(self.builder.block_params(next_block)[0], ty),
            TypeLayout::Void => RValue::Void,
            TypeLayout::Never => return Break(UnreachableCode),
        };

        Continue(if_value)
    }
}
