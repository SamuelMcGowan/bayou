use crate::compiler::ModuleContext;
use crate::ir::ir::*;
use crate::ir::{BinOp, UnOp};

// TODO: make `Spanned` type
pub enum TypeError {
    TypeMismatch { expected: Type, found: Type },
    MissingReturn(Type),
}

pub struct TypeChecker<'a> {
    context: &'a mut ModuleContext,
    errors: Vec<TypeError>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(context: &'a mut ModuleContext) -> Self {
        Self {
            context,
            errors: vec![],
        }
    }

    pub fn run(mut self, module: &mut Module) -> Vec<TypeError> {
        for item in &mut module.items {
            match item {
                Item::FuncDecl(func_decl) => {
                    self.check_func_decl(func_decl);
                }
            }
        }

        self.errors
    }

    fn check_func_decl(&mut self, func_decl: &mut FuncDecl) {
        let expected_ret = Type::I64;

        for stmt in &mut func_decl.statements {
            match stmt {
                Stmt::Assign { local, expr } => {
                    self.check_expr(expr);

                    let local_ty = self.context.symbols.locals[*local].ty;
                    if let Some(ty) = expr.ty {
                        self.check_types_match(local_ty, ty);
                    }
                }

                Stmt::Return(expr) => {
                    self.check_expr(expr);
                    if let Some(ty) = expr.ty {
                        self.check_types_match(expected_ret, ty);
                    }
                }
            }
        }

        // If the function needs to return a value, ensure that the final statement returns.
        // Return *types* are already checked.
        if expected_ret != Type::Void
            && !matches!(func_decl.statements.last(), Some(stmt) if stmt.always_returns())
        {
            self.errors.push(TypeError::MissingReturn(expected_ret));
        }
    }

    fn check_expr(&mut self, expr: &mut Expr) {
        expr.ty = match &mut expr.kind {
            ExprKind::Constant(constant) => Some(constant.ty()),

            ExprKind::Var(local) => Some(self.context.symbols.locals[*local].ty),

            ExprKind::UnOp { op, expr } => {
                self.check_expr(expr);

                match op {
                    UnOp::Negate => expr.ty.map(|ty| {
                        self.check_types_match(Type::I64, ty);
                        Type::I64
                    }),
                    UnOp::BitwiseInvert => expr.ty.map(|ty| {
                        self.check_types_match(Type::I64, ty);
                        Type::I64
                    }),
                }
            }

            ExprKind::BinOp { op, lhs, rhs } => {
                self.check_expr(lhs);
                self.check_expr(rhs);

                let (exp_lhs, exp_rhs, out) = match op {
                    BinOp::Add => (Type::I64, Type::I64, Type::I64),
                    BinOp::Sub => (Type::I64, Type::I64, Type::I64),
                    BinOp::Mul => (Type::I64, Type::I64, Type::I64),
                    BinOp::Div => (Type::I64, Type::I64, Type::I64),
                    BinOp::Mod => (Type::I64, Type::I64, Type::I64),
                    BinOp::BitwiseAnd => (Type::I64, Type::I64, Type::I64),
                    BinOp::BitwiseOr => (Type::I64, Type::I64, Type::I64),
                    BinOp::BitwiseXor => (Type::I64, Type::I64, Type::I64),
                };

                if let Some(ty) = lhs.ty {
                    self.check_types_match(exp_lhs, ty);
                }

                if let Some(ty) = rhs.ty {
                    self.check_types_match(exp_rhs, ty);
                }

                Some(out)
            }
        };
    }

    fn check_types_match(&mut self, expected: Type, found: Type) {
        let types_match = match (expected, found) {
            (_, Type::Never) => true,
            (Type::Never, _) => false,
            (a, b) => a == b,
        };

        if !types_match {
            self.errors
                .push(TypeError::TypeMismatch { expected, found });
        }
    }
}

impl Stmt {
    fn always_returns(&self) -> bool {
        match self {
            Self::Return(_) => true,
            Self::Assign { .. } => false,
        }
    }
}
