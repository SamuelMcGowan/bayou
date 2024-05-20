use bayou_ir::ir::*;
use bayou_ir::symbols::FuncId;
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
        let (block_type, block_type_span) =
            self.check_block_expr(&mut func_decl.block, func_decl.id);

        // FIXME: use return type span
        let func_symbol = &self.module_cx.symbols.funcs[func_decl.id];
        if let Some(block_type) = block_type {
            self.check_types_match(
                func_decl.ret_ty,
                Some(func_symbol.ident.span),
                block_type,
                block_type_span,
            );
        }
    }

    // FIXME: create a FuncTypeChecker struct that stores function information so we don't have to pass it around.
    fn check_stmt(&mut self, stmt: &mut Stmt, func_id: FuncId) {
        match stmt {
            Stmt::Assign { local, expr } => {
                self.check_expr(expr, func_id);

                let local = &self.module_cx.symbols.locals[*local];
                if let Some(ty) = expr.ty {
                    self.check_types_match(local.ty, Some(local.ty_span), ty, expr.span);
                }
            }

            Stmt::Drop(expr) => {
                self.check_expr(expr, func_id);
            }

            Stmt::Return(expr) => {
                self.check_expr(expr, func_id);
                if let Some(ty) = expr.ty {
                    // FIXME: use return type span
                    let func_symbol = &self.module_cx.symbols.funcs[func_id];
                    self.check_types_match(
                        func_symbol.ret_ty,
                        Some(func_symbol.ident.span),
                        ty,
                        expr.span,
                    );
                }
            }
        }
    }

    // TODO: split into separate functions.
    fn check_expr(&mut self, expr: &mut Expr, func_id: FuncId) {
        expr.ty = match &mut expr.kind {
            ExprKind::Constant(constant) => Some(constant.ty()),

            ExprKind::Var(local) => Some(self.module_cx.symbols.locals[*local].ty),

            ExprKind::UnOp { op, expr } => {
                self.check_expr(expr, func_id);

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
                self.check_expr(lhs, func_id);
                self.check_expr(rhs, func_id);

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

            ExprKind::Block(block) => self.check_block_expr(block, func_id).0,

            ExprKind::If { cond, then, else_ } => {
                self.check_expr(cond, func_id);

                if let Some(ty) = cond.ty {
                    self.check_types_match(Type::Bool, None, ty, cond.span);
                }

                self.check_expr(then, func_id);

                'check: {
                    if let Some(else_) = else_ {
                        self.check_expr(else_, func_id);

                        if let (Some(then_ty), Some(else_ty)) = (then.ty, else_.ty) {
                            match (then_ty, else_ty) {
                                // If one side is never, assume the other side is the expected type
                                (Type::Never, ty) | (ty, Type::Never) => {
                                    break 'check Some(ty);
                                }

                                (a, b) if a == b => break 'check Some(a),

                                (a, b) => {
                                    self.errors.push(TypeError::TypeMismatch {
                                        expected: a,
                                        expected_span: Some(then.span),
                                        found: b,
                                        found_span: else_.span,
                                    });

                                    // TODO: do this in more places to avoid cascading type errors.
                                    break 'check Some(a);
                                }
                            }
                        }
                    }

                    None
                }
            }
        };
    }

    fn check_block_expr(&mut self, block: &mut Block, func_id: FuncId) -> (Option<Type>, Span) {
        let mut diverging = false;

        for stmt in &mut block.statements {
            self.check_stmt(stmt, func_id);
            diverging |= stmt_is_diverging(stmt);
        }

        self.check_expr(&mut block.final_expr, func_id);

        if diverging {
            // FIXME: use block span
            (Some(Type::Never), block.final_expr.span)
        } else {
            (block.final_expr.ty, block.final_expr.span)
        }
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

fn stmt_is_diverging(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return(_) => true,
        Stmt::Drop(expr) | Stmt::Assign { expr, .. } => expr.ty == Some(Type::Never),
    }
}
