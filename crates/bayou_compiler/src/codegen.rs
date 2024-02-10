use cranelift::codegen::ir::types::I64;
use cranelift::codegen::verify_function;
use cranelift::prelude::*;
use cranelift_module::{Linkage, Module as _};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use target_lexicon::Triple;

use crate::compiler::ModuleContext;
use crate::ir::ast::*;
use crate::target::UnsupportedTarget;
use crate::{CompilerError, CompilerResult};

pub struct Codegen {
    ctx: codegen::Context,
    builder_ctx: FunctionBuilderContext,
    module: ObjectModule,
}

impl Codegen {
    pub fn new(triple: Triple, name: &str) -> CompilerResult<Self> {
        let mut flag_builder = settings::builder();
        flag_builder.set("is_pic", "true").unwrap();
        flag_builder.set("opt_level", "speed").unwrap();

        let flags = settings::Flags::new(flag_builder);

        let isa = match isa::lookup(triple.clone()) {
            Ok(isa_builder) => isa_builder.finish(flags)?,
            Err(_) => {
                return Err(CompilerError::UnsupportedTarget(
                    UnsupportedTarget::ArchUnsupported(triple.architecture),
                ));
            }
        };

        let module_builder =
            ObjectBuilder::new(isa, name, cranelift_module::default_libcall_names())?;

        let module = ObjectModule::new(module_builder);

        Ok(Self {
            ctx: module.make_context(),
            builder_ctx: FunctionBuilderContext::new(),
            module,
        })
    }

    pub fn compile_module(&mut self, module: &Module, cx: &ModuleContext) -> CompilerResult<()> {
        match &module.item {
            Item::FuncDecl(func_decl) => self.gen_func_decl(func_decl, cx)?,
            Item::ParseError => unreachable!(),
        }

        Ok(())
    }

    pub fn finish(self) -> CompilerResult<ObjectProduct> {
        Ok(self.module.finish())
    }

    fn gen_func_decl(&mut self, func_decl: &FuncDecl, cx: &ModuleContext) -> CompilerResult<()> {
        self.module.clear_context(&mut self.ctx);

        // no parameters, one return value
        self.ctx.func.signature.returns.push(AbiParam::new(I64));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block); // no predecessors

        // function codegen
        let mut func_codegen = FuncCodegen {
            builder,
            module: &mut self.module,
            cx,
        };

        for stmt in &func_decl.statements {
            func_codegen.gen_stmt(stmt);
        }

        func_codegen.builder.finalize();

        verify_function(&self.ctx.func, self.module.isa()).unwrap();

        // declare and define in module (not final)
        let name = cx.interner.resolve(&func_decl.name.ident);
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
    cx: &'a ModuleContext,
}

impl FuncCodegen<'_> {
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign {
                ident: _,
                resolved,
                expr,
            } => {
                let local_id = resolved.unwrap();
                let var = Variable::new(local_id.0);

                let val = self.gen_expr(expr);

                self.builder.declare_var(var, I64);
                self.builder.def_var(var, val);
            }

            Stmt::Return(expr) => {
                let value = self.gen_expr(expr);
                self.builder.ins().return_(&[value]);

                let after_return = self.builder.create_block();
                self.builder.switch_to_block(after_return);
                self.builder.seal_block(after_return); // nothing jumps here, dead code
            }

            Stmt::ParseError => unreachable!(),
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> Value {
        match &expr.kind {
            ExprKind::Constant(n) => self.builder.ins().iconst(I64, *n),

            ExprKind::Var(_, resolved) => {
                let local_id = resolved.unwrap();
                let var = Variable::new(local_id.0);
                self.builder.use_var(var)
            }

            ExprKind::UnOp { op, expr } => {
                let expr = self.gen_expr(expr);

                match op {
                    UnOp::Negate => self.builder.ins().ineg(expr),
                    UnOp::BitwiseInvert => self.builder.ins().bnot(expr),
                }
            }

            ExprKind::BinOp { op, lhs, rhs } => {
                let lhs = self.gen_expr(lhs);
                let rhs = self.gen_expr(rhs);

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

            ExprKind::ParseError => unreachable!(),
        }
    }
}
