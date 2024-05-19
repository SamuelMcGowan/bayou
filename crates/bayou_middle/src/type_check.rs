use bayou_ir::ir::*;
use bayou_ir::{BinOp, Type, UnOp};
use bayou_session::diagnostics::prelude::*;

// TODO: make `Spanned` type
pub enum TypeError {
    TypeMismatch {
        expected: Type,
        expected_span: Option<Span>,

        found: Type,
        found_span: Span,
    },

    MissingReturn {
        ty: Type,
        span: Span,
    },
}

impl IntoDiagnostic for TypeError {
    fn into_diagnostic(self, source_id: SourceId, _interner: &Interner) -> Diagnostic {
        match self {
            TypeError::TypeMismatch {
                expected,
                expected_span,
                found,
                found_span,
            } => {
                let mut diagnostic = Diagnostic::error()
                    .with_message(format!("expected type {expected:?}, found type {found:?}"))
                    .with_snippet(Snippet::primary("unexpected type", source_id, found_span));

                if let Some(expected_span) = expected_span {
                    diagnostic = diagnostic.with_snippet(Snippet::secondary(
                        "expected due to this type",
                        source_id,
                        expected_span,
                    ));
                }

                diagnostic
            }

            TypeError::MissingReturn { ty, span } => Diagnostic::error()
                .with_message(format!(
                    "missing return statement in function that returns type {ty:?}"
                ))
                .with_snippet(Snippet::primary(
                    "expected due to this return type",
                    source_id,
                    span,
                )),
        }
    }
}

pub struct TypeChecker<'m> {
    module_cx: &'m mut ModuleContext,
    errors: Vec<TypeError>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(module_cx: &'a mut ModuleContext) -> Self {
        Self {
            module_cx,
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
        for stmt in &mut func_decl.statements {
            match stmt {
                Stmt::Assign { local, expr } => {
                    self.check_expr(expr);

                    let local = &self.module_cx.symbols.locals[*local];
                    if let Some(ty) = expr.ty {
                        self.check_types_match(local.ty, Some(local.ty_span), ty, expr.span);
                    }
                }

                Stmt::Return(expr) => {
                    self.check_expr(expr);
                    if let Some(ty) = expr.ty {
                        // FIXME: use return type span
                        let name_span = self.module_cx.symbols.funcs[func_decl.id].ident.span;
                        self.check_types_match(func_decl.ret_ty, Some(name_span), ty, expr.span);
                    }
                }
            }
        }

        // If the function needs to return a value, ensure that the final statement returns.
        // Return *types* are already checked.
        if func_decl.ret_ty != Type::Void
            && !matches!(func_decl.statements.last(), Some(stmt) if stmt_always_returns(stmt))
        {
            // FIXME: use return type span
            let name_span = self.module_cx.symbols.funcs[func_decl.id].ident.span;

            self.errors.push(TypeError::MissingReturn {
                ty: func_decl.ret_ty,
                span: name_span,
            });
        }
    }

    fn check_expr(&mut self, expr: &mut Expr) {
        expr.ty = match &mut expr.kind {
            ExprKind::Constant(constant) => Some(constant.ty()),

            ExprKind::Var(local) => Some(self.module_cx.symbols.locals[*local].ty),

            ExprKind::UnOp { op, expr } => {
                self.check_expr(expr);

                match op {
                    UnOp::Negate => expr.ty.map(|ty| {
                        self.check_types_match(Type::I64, None, ty, expr.span);
                        Type::I64
                    }),

                    UnOp::BitwiseInvert => expr.ty.map(|ty| {
                        self.check_types_match(Type::I64, None, ty, expr.span);
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
                    self.check_types_match(exp_lhs, None, ty, lhs.span);
                }

                if let Some(ty) = rhs.ty {
                    self.check_types_match(exp_rhs, None, ty, rhs.span);
                }

                Some(out)
            }
        };
    }

    fn check_types_match(
        &mut self,
        expected: Type,
        expected_span: Option<Span>,
        found: Type,
        found_span: Span,
    ) {
        let types_match = match (expected, found) {
            (_, Type::Never) => true,
            (Type::Never, _) => false,
            (a, b) => a == b,
        };

        if !types_match {
            self.errors.push(TypeError::TypeMismatch {
                expected,
                expected_span,

                found,
                found_span,
            });
        }
    }
}

fn stmt_always_returns(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return(_) => true,
        Stmt::Assign { .. } => false,
    }
}
