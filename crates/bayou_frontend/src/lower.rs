use bayou_ir::symbols::*;
use bayou_ir::{ir, Type};
use bayou_session::diagnostics::prelude::*;
use bayou_session::{Ident, InternedStr};

use crate::ast;

pub enum NameError {
    LocalUndefined(Ident),
    DuplicateGlobal { first: Ident, second: Ident },
}

impl IntoDiagnostic for NameError {
    fn into_diagnostic(self, source_id: SourceId, interner: &Interner) -> Diagnostic {
        match self {
            Self::DuplicateGlobal { first, second } => {
                let ident_str = interner.get(first.ident_str);
                Diagnostic::error()
                    .with_message(format!("duplicate global `{ident_str}`"))
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

            Self::LocalUndefined(ident) => {
                let ident_str = interner.get(ident.ident_str);
                Diagnostic::error()
                    .with_message(format!("undefined variable `{ident_str}`"))
                    .with_snippet(Snippet::primary(
                        "undefined variable here",
                        source_id,
                        ident.span,
                    ))
            }
        }
    }
}

struct LocalEntry {
    ident_str: InternedStr,
    id: LocalId,
}

pub struct Lowerer {
    symbols: Symbols,
    errors: Vec<NameError>,

    local_stack: Vec<LocalEntry>,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            symbols: Symbols::default(),
            errors: vec![],

            local_stack: vec![],
        }
    }

    pub fn run(mut self, module: ast::Module) -> Result<(ir::Module, Symbols), Vec<NameError>> {
        self.declare_globals(&module.items);

        let module = self.lower_module(module);
        if self.errors.is_empty() {
            Ok((module, self.symbols))
        } else {
            Err(self.errors)
        }
    }

    fn declare_globals(&mut self, items: &[ast::Item]) {
        for item in items {
            match item {
                ast::Item::FuncDecl(func_decl) => self.declare_global_func(FunctionSymbol {
                    ident: func_decl.ident,
                    ret_ty: func_decl.ret_ty,
                }),

                ast::Item::ParseError => {}
            }
        }
    }

    fn declare_global_func(&mut self, symbol: FunctionSymbol) {
        let ident = symbol.ident;

        let func_id = self.symbols.funcs.insert(symbol);

        if let Some(first_symbol) = self
            .symbols
            .global_lookup
            .insert(ident.ident_str, GlobalId::Func(func_id))
        {
            self.errors.push(NameError::DuplicateGlobal {
                first: self.symbols.get_global_ident(first_symbol).unwrap(),
                second: ident,
            })
        }
    }

    fn lower_module(&mut self, module: ast::Module) -> ir::Module {
        let mut items_lowered = vec![];

        for item in module.items {
            match item {
                ast::Item::FuncDecl(func_decl) => {
                    if let Some(func_decl) = self.lower_func_decl(func_decl) {
                        items_lowered.push(ir::Item::FuncDecl(func_decl));
                    }
                }
                ast::Item::ParseError => {}
            }
        }

        ir::Module {
            items: items_lowered,
        }
    }

    fn lower_func_decl(&mut self, func_decl: ast::FuncDecl) -> Option<ir::FuncDecl> {
        self.clear_locals();

        // no parameters for now

        let block = self.lower_block_expr(func_decl.block)?;

        // TODO: is this lookup necessary
        let id = self.symbols.global_lookup[&func_decl.ident.ident_str]
            .as_func()
            .unwrap();

        Some(ir::FuncDecl {
            id,
            ret_ty: func_decl.ret_ty,
            block,
        })
    }

    fn lower_expr(&mut self, expr: ast::Expr) -> Option<ir::Expr> {
        let expr_kind = match expr.kind {
            ast::ExprKind::Integer(n) => ir::ExprKind::Constant(ir::Constant::I64(n)),
            ast::ExprKind::Bool(b) => ir::ExprKind::Constant(ir::Constant::Bool(b)),

            ast::ExprKind::Var(ident) => {
                let id = self.lookup_local(ident)?;
                ir::ExprKind::Var(id)
            }

            ast::ExprKind::UnOp { op, expr } => {
                let expr = self.lower_expr(*expr)?;
                ir::ExprKind::UnOp {
                    op,
                    expr: Box::new(expr),
                }
            }

            ast::ExprKind::BinOp { op, lhs, rhs } => {
                // lower both before using `?`
                let lhs = self.lower_expr(*lhs);
                let rhs = self.lower_expr(*rhs);

                ir::ExprKind::BinOp {
                    op,
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                }
            }

            ast::ExprKind::Block(block) => {
                let lowered_block = self.lower_block_expr(*block)?;
                ir::ExprKind::Block(Box::new(lowered_block))
            }

            ast::ExprKind::If { cond, then, else_ } => {
                let cond = self.lower_expr(*cond);

                let then = self.in_scope(|s| s.lower_expr(*then));
                let else_ = self.in_scope(|s| else_.map(|e| s.lower_expr(*e)));

                ir::ExprKind::If {
                    cond: Box::new(cond?),
                    then: Box::new(then?),
                    else_: match else_ {
                        Some(e) => Some(Box::new(e?)),
                        None => None,
                    },
                }
            }

            ast::ExprKind::Void => ir::ExprKind::Constant(ir::Constant::Void),

            ast::ExprKind::ParseError => return None,
        };

        Some(ir::Expr {
            kind: expr_kind,
            span: expr.span,
            ty: None,
        })
    }

    fn lower_block_expr(&mut self, block: ast::Block) -> Option<ir::Block> {
        self.in_scope(|lowerer| {
            let mut lowered_stmts = vec![];

            for stmt in block.statements {
                match stmt {
                    ast::Stmt::Assign { ident, ty, expr } => {
                        let expr = lowerer.lower_expr(expr);
                        let local_id = lowerer.declare_local(ident, ty);

                        if let Some(expr) = expr {
                            lowered_stmts.push(ir::Stmt::Assign {
                                local: local_id,
                                expr,
                            })
                        }
                    }

                    ast::Stmt::Drop(expr) => {
                        if let Some(expr) = lowerer.lower_expr(expr) {
                            lowered_stmts.push(ir::Stmt::Drop(expr));
                        }
                    }

                    ast::Stmt::Return(expr) => {
                        if let Some(expr) = lowerer.lower_expr(expr) {
                            lowered_stmts.push(ir::Stmt::Return(expr));
                        }
                    }

                    ast::Stmt::ParseError => {}
                }
            }

            let lowered_final_expr = lowerer.lower_expr(block.final_expr)?;

            Some(ir::Block {
                statements: lowered_stmts,
                final_expr: lowered_final_expr,
            })
        })
    }

    #[must_use]
    fn declare_local(&mut self, ident: Ident, ty: Type) -> LocalId {
        let id = self.symbols.locals.insert(LocalSymbol {
            ident,
            ty,
            // FIXME: use variable type span
            ty_span: ident.span,
        });

        self.local_stack.push(LocalEntry {
            ident_str: ident.ident_str,
            id,
        });

        id
    }

    fn lookup_local(&mut self, ident: Ident) -> Option<LocalId> {
        let id = self
            .local_stack
            .iter()
            .rev()
            .find_map(|entry| (entry.ident_str == ident.ident_str).then_some(entry.id));

        if id.is_none() {
            self.errors.push(NameError::LocalUndefined(ident));
        }

        id
    }

    fn clear_locals(&mut self) {
        self.local_stack.clear();
    }

    fn in_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let scope = self.start_scope();
        let output = f(self);
        self.end_scope(scope);
        output
    }

    #[must_use]
    fn start_scope(&self) -> usize {
        self.local_stack.len()
    }

    fn end_scope(&mut self, start: usize) {
        self.local_stack.truncate(start);
    }
}
