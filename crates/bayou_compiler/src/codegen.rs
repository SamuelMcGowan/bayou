use cranelift::codegen::ir::types::I64;
use cranelift::prelude::*;
use cranelift_module::{Linkage, Module as _};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use target_lexicon::Triple;

use crate::compiler::ModuleContext;
use crate::ir::ast::*;

#[derive(thiserror::Error, Debug)]
pub enum CodegenError {
    #[error(transparent)]
    Module(#[from] cranelift_module::ModuleError),

    #[error(transparent)]
    Codegen(#[from] cranelift::codegen::CodegenError),

    #[error("bad target {0}: {1}")]
    BadTarget(Triple, cranelift::codegen::isa::LookupError),
}

pub type CodegenResult<T> = Result<T, CodegenError>;

pub struct Codegen<'a> {
    ctx: codegen::Context,
    builder_ctx: FunctionBuilderContext,
    module: ObjectModule,

    module_cx: &'a ModuleContext,
}

impl<'a> Codegen<'a> {
    pub fn new(triple: Triple, name: &str, module_cx: &'a ModuleContext) -> CodegenResult<Self> {
        let mut flag_builder = settings::builder();
        flag_builder.set("is_pic", "true").unwrap();
        flag_builder.set("opt_level", "speed").unwrap();

        let flags = settings::Flags::new(flag_builder);

        let isa = match isa::lookup(triple.clone()) {
            Ok(isa_builder) => isa_builder.finish(flags)?,
            Err(err) => return Err(CodegenError::BadTarget(triple, err)),
        };

        let module_builder =
            ObjectBuilder::new(isa, name, cranelift_module::default_libcall_names())?;

        let module = ObjectModule::new(module_builder);

        Ok(Self {
            ctx: module.make_context(),
            builder_ctx: FunctionBuilderContext::new(),
            module,

            module_cx,
        })
    }

    pub fn run(mut self, module: &Module) -> CodegenResult<ObjectProduct> {
        match &module.item {
            Item::FuncDecl(func_decl) => self.gen_func_decl(func_decl)?,
            Item::ParseError => unreachable!(),
        }

        Ok(self.module.finish())
    }

    fn gen_func_decl(&mut self, func_decl: &FuncDecl) -> CodegenResult<()> {
        self.module.clear_context(&mut self.ctx);

        // no parameters, one return value
        self.ctx.func.signature.returns.push(AbiParam::new(I64));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

        let entry_block = builder.create_block();

        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block); // block can't be jumped to

        // function codegen
        let mut func_codegen = FuncCodegen {
            builder,
            module: &mut self.module,
        };
        func_codegen.gen_stmt(&func_decl.statement);
        func_codegen.builder.finalize();

        // declare and define in module
        let name = self.module_cx.interner.resolve(&func_decl.name.ident);
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
}

impl FuncCodegen<'_> {
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return(expr) => {
                let value = self.gen_expr(expr);
                self.builder.ins().return_(&[value]);
            }
            Stmt::ParseError => unreachable!(),
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> Value {
        match &expr.kind {
            ExprKind::Constant(n) => {
                // FIXME: `n` should already be an i64
                self.builder.ins().iconst(I64, i64::try_from(*n).unwrap())
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
