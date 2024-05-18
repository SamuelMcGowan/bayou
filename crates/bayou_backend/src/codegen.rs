use bayou_ir::ir::*;
use bayou_ir::{BinOp, UnOp};
use bayou_session::diagnostics::DiagnosticEmitter;
use bayou_session::Session;
use cranelift::codegen::ir::types::{I64, I8};
use cranelift::codegen::verify_function;
use cranelift::prelude::*;
use cranelift_module::{Linkage, Module as _};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use target_lexicon::Triple;

use crate::{BackendError, BackendResult};

type IrType = bayou_ir::Type;

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

        match func_decl.ret_ty {
            IrType::I64 => {
                self.ctx.func.signature.returns.push(AbiParam::new(I64));
            }
            IrType::Bool => self.ctx.func.signature.returns.push(AbiParam::new(I8)),
            IrType::Void | IrType::Never => {}
        }

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block); // no predecessors

        // function codegen
        let mut func_codegen = FuncCodegen {
            builder,
            module: &mut self.module,
            module_cx,
        };

        for stmt in &func_decl.statements {
            func_codegen.gen_stmt(stmt);
        }

        func_codegen.builder.finalize();

        verify_function(&self.ctx.func, self.module.isa()).unwrap();

        // declare and define in module (not final)
        let ident = module_cx.symbols.funcs[func_decl.id].ident;
        let name = self.session.interner.get(ident.ident_str);
        let id = self
            .module
            .declare_function(name, Linkage::Export, &self.ctx.func.signature)?;
        self.module.define_function(id, &mut self.ctx)?;

        Ok(())
    }
}

struct FuncCodegen<'a> {
    builder: FunctionBuilder<'a>,
    module: &'a mut ObjectModule,
    module_cx: &'a ModuleContext,
}

impl FuncCodegen<'_> {
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { local, expr } => {
                let var = Variable::new(local.0);

                let val = self.gen_expr(expr);

                let ty = self.module_cx.symbols.locals[*local].ty;
                let ty = match ty {
                    IrType::I64 => Some(I64),
                    IrType::Bool => Some(I8),
                    IrType::Void | IrType::Never => None,
                };

                // void types are not declared
                if let (Some(val), Some(ty)) = (val, ty) {
                    self.builder.declare_var(var, ty);
                    self.builder.def_var(var, val);
                }
            }

            Stmt::Return(expr) => {
                let value = self.gen_expr(expr);

                if let Some(value) = value {
                    self.builder.ins().return_(&[value]);
                } else {
                    self.builder.ins().return_(&[]);
                }

                let after_return = self.builder.create_block();
                self.builder.switch_to_block(after_return);
                self.builder.seal_block(after_return); // nothing jumps here, dead code
            }
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> Option<Value> {
        let value = match &expr.kind {
            ExprKind::Constant(constant) => match constant {
                Constant::I64(n) => self.builder.ins().iconst(I64, *n),
                Constant::Bool(b) => self.builder.ins().iconst(I8, if *b { 1 } else { 0 }),
            },

            ExprKind::Var(local) => {
                let local_ty = self.module_cx.symbols.locals[*local].ty;

                // Void typed variables don't emit expressions.
                if local_ty == IrType::Void {
                    return None;
                }

                let var = Variable::new(local.0);
                self.builder.use_var(var)
            }

            ExprKind::UnOp { op, expr } => {
                let expr = self.gen_expr(expr).unwrap();

                match op {
                    UnOp::Negate => self.builder.ins().ineg(expr),
                    UnOp::BitwiseInvert => self.builder.ins().bnot(expr),
                }
            }

            ExprKind::BinOp { op, lhs, rhs } => {
                let lhs = self.gen_expr(lhs).unwrap();
                let rhs = self.gen_expr(rhs).unwrap();

                let ins = self.builder.ins();
                match op {
                    BinOp::Add => ins.iadd(lhs, rhs),
                    BinOp::Sub => ins.isub(lhs, rhs),
                    BinOp::Mul => ins.imul(lhs, rhs),
                    BinOp::Div => ins.sdiv(lhs, rhs),
                    BinOp::Mod => ins.srem(lhs, rhs),
                    BinOp::BitwiseAnd => ins.band(lhs, rhs),
                    BinOp::BitwiseOr => ins.bor(lhs, rhs),
                    BinOp::BitwiseXor => ins.bxor(lhs, rhs),
                }
            }

            ExprKind::Void => return None,
        };

        Some(value)
    }
}
