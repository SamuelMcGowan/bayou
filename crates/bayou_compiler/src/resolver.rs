use crate::compiler::ModuleContext;
use crate::ir::ir::Type;
use crate::ir::{ast, ir, Ident, InternedStr};
use crate::symbols::{GlobalSymbol, LocalId, LocalSymbol};

pub enum ResolverError {
    LocalUndefined(Ident),
    DuplicateGlobal { first: Ident, second: Ident },
}

pub struct Resolver<'cx> {
    context: &'cx mut ModuleContext,
    errors: Vec<ResolverError>,

    local_stack: Vec<LocalEntry>,
}

impl<'cx> Resolver<'cx> {
    pub fn new(context: &'cx mut ModuleContext) -> Self {
        Self {
            context,
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
                ast::Item::FuncDecl(func_decl) => self.declare_global(GlobalSymbol {
                    ident: func_decl.name,
                }),
                ast::Item::ParseError => unreachable!(),
            }
        }
    }

    fn declare_global(&mut self, symbol: GlobalSymbol) {
        let ident = symbol.ident;

        if let Some(first_symbol) = self
            .context
            .symbols
            .globals
            .insert(symbol.ident.ident, symbol)
        {
            self.errors.push(ResolverError::DuplicateGlobal {
                first: first_symbol.ident,
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
                ast::Item::ParseError => unreachable!(),
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

                ast::Stmt::ParseError => unreachable!(),
            }
        }

        Some(ir::FuncDecl {
            name: func_decl.name,
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

            ast::ExprKind::ParseError => unreachable!(),
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
        let id = self.context.symbols.locals.insert(LocalSymbol {
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
