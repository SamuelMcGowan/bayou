use bayou_interner::{Interner, Istr};
use bayou_ir::symbols::*;
use bayou_ir::{ir, Ident, IdentWithSource, Type};
use bayou_session::diagnostics::prelude::*;
use bayou_session::sourcemap::SourceSpan;

use crate::ast;
use crate::gather_modules::ParsedModule;
use crate::module_tree::ModuleTree;

pub enum NameError {
    LocalUndefined(IdentWithSource),

    DuplicateGlobal {
        first: IdentWithSource,
        second: IdentWithSource,
    },
}

impl IntoDiagnostic<Interner> for NameError {
    fn into_diagnostic(self, interner: &Interner) -> Diagnostic {
        match self {
            Self::DuplicateGlobal { first, second } => {
                let ident_str = &interner[first.istr];

                Diagnostic::error()
                    .with_message(format!("duplicate global `{ident_str}`"))
                    .with_snippet(Snippet::secondary(
                        "first definition",
                        first.span.source_id,
                        first.span.span,
                    ))
                    .with_snippet(Snippet::primary(
                        "second definition",
                        second.span.source_id,
                        second.span.span,
                    ))
            }

            Self::LocalUndefined(ident) => {
                let ident_str = &interner[ident.istr];
                Diagnostic::error()
                    .with_message(format!("undefined variable `{ident_str}`"))
                    .with_snippet(Snippet::primary(
                        "undefined variable here",
                        ident.span.source_id,
                        ident.span.span,
                    ))
            }
        }
    }
}

struct LocalEntry {
    ident_str: Istr,
    id: LocalId,
}

pub struct ModuleLowerer<'a, 'b> {
    module: &'a ParsedModule,
    module_tree: &'a ModuleTree,

    symbols: &'b mut Symbols,
    package_ir: &'b mut ir::PackageIr,
    errors: &'b mut Vec<NameError>,

    local_stack: Vec<LocalEntry>,
}

impl<'a, 'b> ModuleLowerer<'a, 'b> {
    pub fn new(
        module: &'a ParsedModule,
        module_tree: &'a ModuleTree,
        symbols: &'b mut Symbols,
        package_ir: &'b mut ir::PackageIr,
        errors: &'b mut Vec<NameError>,
    ) -> Self {
        Self {
            module,
            module_tree,

            symbols,
            package_ir,
            errors,

            local_stack: vec![],
        }
    }

    pub fn run(mut self) {
        self.declare_globals();
        self.lower_module()
    }

    fn declare_globals(&mut self) {
        for item in &self.module.ast.items {
            match item {
                ast::Item::FuncDecl(func_decl) => self.declare_global_func(FunctionSymbol {
                    ident: func_decl.ident.with_source(self.module.source_id),

                    ret_ty: func_decl.ret_ty,
                    ret_ty_span: SourceSpan::new(func_decl.ret_ty_span, self.module.source_id),
                }),

                ast::Item::Submodule(_) | ast::Item::ParseError => {}
            }
        }
    }

    fn declare_global_func(&mut self, symbol: FunctionSymbol) {
        let ident = symbol.ident;

        let func_id = self.symbols.funcs.insert(symbol);

        if let Some(first_symbol) = self
            .symbols
            .global_lookup
            .insert(ident.istr, GlobalId::Func(func_id))
        {
            self.errors.push(NameError::DuplicateGlobal {
                first: self.symbols.get_global_ident(first_symbol).unwrap(),
                second: ident,
            })
        }
    }

    fn lower_module(&mut self) {
        for item in &self.module.ast.items {
            match item {
                ast::Item::FuncDecl(func_decl) => {
                    if let Some(func_decl) = self.lower_func_decl(func_decl) {
                        self.package_ir.items.push(ir::Item::FuncDecl(func_decl));
                    }
                }
                ast::Item::Submodule(_) | ast::Item::ParseError => {}
            }
        }
    }

    fn lower_func_decl(&mut self, func_decl: &ast::FuncDecl) -> Option<ir::FuncDecl> {
        self.clear_locals();

        // no parameters for now

        let block = self.lower_block_expr(&func_decl.block)?;

        let id = self.symbols.global_lookup[&func_decl.ident.istr]
            .as_func()
            .unwrap();

        Some(ir::FuncDecl { id, block })
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> Option<ir::Expr> {
        let expr_kind = match &expr.kind {
            ast::ExprKind::Integer(n) => ir::ExprKind::Constant(ir::Constant::I64(*n)),
            ast::ExprKind::Bool(b) => ir::ExprKind::Constant(ir::Constant::Bool(*b)),

            ast::ExprKind::Var(ident) => {
                let id = self.lookup_local(*ident)?;
                ir::ExprKind::Var(id)
            }

            ast::ExprKind::UnOp { op, expr } => {
                let expr = self.lower_expr(expr)?;
                ir::ExprKind::UnOp {
                    op: *op,
                    expr: Box::new(expr),
                }
            }

            ast::ExprKind::BinOp { op, lhs, rhs } => {
                // lower both before using `?`
                let lhs = self.lower_expr(lhs);
                let rhs = self.lower_expr(rhs);

                ir::ExprKind::BinOp {
                    op: *op,
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                }
            }

            ast::ExprKind::Block(block) => {
                let lowered_block = self.lower_block_expr(block)?;
                ir::ExprKind::Block(Box::new(lowered_block))
            }

            ast::ExprKind::If { cond, then, else_ } => {
                let cond = self.lower_expr(cond);

                let then = self.in_scope(|s| s.lower_expr(then));
                let else_ = self.in_scope(|s| else_.as_ref().map(|e| s.lower_expr(e)));

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
            span: SourceSpan::new(expr.span, self.module.source_id),
            ty: None,
        })
    }

    fn lower_block_expr(&mut self, block: &ast::Block) -> Option<ir::Block> {
        self.in_scope(|lowerer| {
            let mut lowered_stmts = vec![];

            for stmt in &block.statements {
                match stmt {
                    ast::Stmt::Assign { ident, ty, expr } => {
                        let expr = lowerer.lower_expr(expr);
                        let local_id = lowerer.declare_local(*ident, *ty);

                        if let Some(expr) = expr {
                            lowered_stmts.push(ir::Stmt::Assign {
                                local: local_id,
                                expr,
                            })
                        }
                    }

                    ast::Stmt::Drop {
                        expr,
                        had_semicolon: _,
                    } => {
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

            let lowered_final_expr = lowerer.lower_expr(&block.final_expr)?;

            Some(ir::Block {
                statements: lowered_stmts,
                final_expr: lowered_final_expr,

                span: SourceSpan::new(block.span, lowerer.module.source_id),
            })
        })
    }

    #[must_use]
    fn declare_local(&mut self, ident: Ident, ty: Type) -> LocalId {
        let ident = ident.with_source(self.module.source_id);

        let id = self.symbols.locals.insert(LocalSymbol {
            ident,
            ty,
            ty_span: ident.span,
        });

        self.local_stack.push(LocalEntry {
            ident_str: ident.istr,
            id,
        });

        id
    }

    fn lookup_local(&mut self, ident: Ident) -> Option<LocalId> {
        let id = self
            .local_stack
            .iter()
            .rev()
            .find_map(|entry| (entry.ident_str == ident.istr).then_some(entry.id));

        if id.is_none() {
            self.errors.push(NameError::LocalUndefined(
                ident.with_source(self.module.source_id),
            ));
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
