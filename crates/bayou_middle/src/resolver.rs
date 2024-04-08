use bayou_frontend::ast;
use bayou_ir::symbols::{FunctionSymbol, GlobalId, LocalId, LocalSymbol};
use bayou_ir::{ir, Ident, Type};
use bayou_session::diagnostics::prelude::*;

use crate::ModuleCompilation;

pub enum ResolverError {
    LocalUndefined(Ident),
    DuplicateGlobal { first: Ident, second: Ident },
}

impl IntoDiagnostic for ResolverError {
    fn into_diagnostic(self, source_id: SourceId, interner: &Interner) -> Diagnostic {
        match self {
            ResolverError::DuplicateGlobal { first, second } => {
                let name_str = interner.resolve(&first.ident);
                Diagnostic::error()
                    .with_message(format!("duplicate global `{name_str}`"))
                    .with_snippet(Snippet::secondary(
                        "first definition",
                        source_id,
                        first.span,
                    ))
                    .with_snippet(Snippet::primary(
                        "second definition",
                        source_id,
                        second.span,
                    ))
            }

            ResolverError::LocalUndefined(ident) => {
                let name_str = interner.resolve(&ident.ident);
                Diagnostic::error()
                    .with_message(format!("undefined variable `{name_str}`"))
                    .with_snippet(Snippet::primary(
                        "undefined variable here",
                        source_id,
                        ident.span,
                    ))
            }
        }
    }
}

pub struct Resolver<'m> {
    compilation: &'m mut ModuleCompilation,
    errors: Vec<ResolverError>,

    local_stack: Vec<LocalEntry>,
}

impl<'m> Resolver<'m> {
    pub fn new(compilation: &'m mut ModuleCompilation) -> Self {
        Self {
            compilation,
            errors: vec![],

            local_stack: vec![],
        }
    }

    pub fn run(mut self, module: ast::Module) -> Result<ir::Module, Vec<ResolverError>> {
        self.declare_globals(&module.items);

        let module = self.resolve(module);
        if self.errors.is_empty() {
            Ok(module)
        } else {
            Err(self.errors)
        }
    }

    fn declare_globals(&mut self, items: &[ast::Item]) {
        for item in items {
            match item {
                ast::Item::FuncDecl(func_decl) => self.declare_global_func(FunctionSymbol {
                    ident: func_decl.name,

                    ret_ty: func_decl.ret_ty,
                }),
                ast::Item::ParseError => {}
            }
        }
    }

    fn declare_global_func(&mut self, symbol: FunctionSymbol) {
        let ident = symbol.ident;

        let func_id = self.compilation.symbols.funcs.insert(symbol);

        if let Some(first_symbol) = self
            .compilation
            .symbols
            .global_lookup
            .insert(ident.ident, GlobalId::Func(func_id))
        {
            self.errors.push(ResolverError::DuplicateGlobal {
                first: self
                    .compilation
                    .symbols
                    .get_global_ident(first_symbol)
                    .unwrap(),
                second: ident,
            })
        }
    }

    fn resolve(&mut self, module: ast::Module) -> ir::Module {
        let mut items_resolved = vec![];

        for item in module.items {
            match item {
                ast::Item::FuncDecl(func_decl) => {
                    if let Some(func_decl) = self.resolve_func_decl(func_decl) {
                        items_resolved.push(ir::Item::FuncDecl(func_decl));
                    }
                }
                ast::Item::ParseError => {}
            }
        }

        ir::Module {
            items: items_resolved,
        }
    }

    fn resolve_func_decl(&mut self, func_decl: ast::FuncDecl) -> Option<ir::FuncDecl> {
        self.clear_locals();

        // no parameters for now

        let mut resolved_stmts = vec![];
        for stmt in func_decl.statements {
            match stmt {
                ast::Stmt::Assign { ident, expr } => {
                    let expr = self.resolve_expr(expr);
                    let local_id = self.declare_local(ident, Type::I64);

                    if let Some(expr) = expr {
                        resolved_stmts.push(ir::Stmt::Assign {
                            local: local_id,
                            expr,
                        })
                    }
                }

                ast::Stmt::Return(expr) => {
                    if let Some(expr) = self.resolve_expr(expr) {
                        resolved_stmts.push(ir::Stmt::Return(expr));
                    }
                }

                ast::Stmt::ParseError => {}
            }
        }

        Some(ir::FuncDecl {
            name: func_decl.name,
            ret_ty: func_decl.ret_ty,
            statements: resolved_stmts,
        })
    }

    fn resolve_expr(&mut self, expr: ast::Expr) -> Option<ir::Expr> {
        let expr_kind = match expr.kind {
            ast::ExprKind::Constant(n) => ir::ExprKind::Constant(ir::Constant::I64(n)),

            ast::ExprKind::Var(ident) => {
                let id = self.lookup_local(ident)?;
                ir::ExprKind::Var(id)
            }

            ast::ExprKind::UnOp { op, expr } => {
                let expr = self.resolve_expr(*expr)?;
                ir::ExprKind::UnOp {
                    op,
                    expr: Box::new(expr),
                }
            }

            ast::ExprKind::BinOp { op, lhs, rhs } => {
                // resolve both before using `?`
                let lhs = self.resolve_expr(*lhs);
                let rhs = self.resolve_expr(*rhs);

                ir::ExprKind::BinOp {
                    op,
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                }
            }

            ast::ExprKind::Void => ir::ExprKind::Void,

            ast::ExprKind::ParseError => return None,
        };

        Some(ir::Expr {
            kind: expr_kind,
            span: expr.span,
            ty: None,
        })
    }

    fn clear_locals(&mut self) {
        self.local_stack.clear();
    }

    #[must_use]
    fn start_scope(&self) -> usize {
        self.local_stack.len()
    }

    fn end_scope(&mut self, start: usize) {
        self.local_stack.truncate(start);
    }

    #[must_use]
    fn declare_local(&mut self, ident: Ident, ty: Type) -> LocalId {
        let id = self.compilation.symbols.locals.insert(LocalSymbol {
            ident,
            ty,
            // FIXME: use variable type span
            ty_span: ident.span,
        });

        self.local_stack.push(LocalEntry {
            ident: ident.ident,
            id,
        });

        id
    }

    fn lookup_local(&mut self, ident: Ident) -> Option<LocalId> {
        let id = self
            .local_stack
            .iter()
            .rev()
            .find_map(|entry| (entry.ident == ident.ident).then_some(entry.id));

        if id.is_none() {
            self.errors.push(ResolverError::LocalUndefined(ident));
        }

        id
    }
}

struct LocalEntry {
    ident: InternedStr,
    id: LocalId,
}
