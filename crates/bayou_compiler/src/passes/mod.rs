pub mod type_check;
pub mod entry_point;

// use std::ops::ControlFlow;

// use crate::ir::ir::*;
// use crate::symbols::LocalId;

// /// Visitor functions may return [`ControlFlow::Break`] in order
// /// to avoid children being walked.
// #[allow(unused_variables)]
// pub trait Visitor {
//     fn visit_module(&mut self, module: &Module) -> ControlFlow<()> {
//         ControlFlow::Continue(())
//     }

//     fn visit_item(&mut self, item: &Item) -> ControlFlow<()> {
//         ControlFlow::Continue(())
//     }

//     fn visit_func_decl(&mut self, func_decl: &FuncDecl) -> ControlFlow<()> {
//         ControlFlow::Continue(())
//     }

//     fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
//         ControlFlow::Continue(())
//     }

//     fn visit_stmt_assign(&mut self, local: LocalId, expr: &Expr) -> ControlFlow<()> {
//         ControlFlow::Continue(())
//     }

//     fn visit_stmt_return(&mut self, expr: &Expr) -> ControlFlow<()> {
//         ControlFlow::Continue(())
//     }

//     fn visit_expr(&mut self, expr: &Expr) {}
// }

// pub fn walk<V: Visitor>(visitor: &mut V, module: &Module) {
//     if visitor.visit_module(module).is_continue() {
//         for item in &module.items {
//             if visitor.visit_item(item).is_continue() {
//                 match item {
//                     Item::FuncDecl(func_decl) => {
//                         if visitor.visit_func_decl(func_decl).is_continue() {
//                             walk_func_decl(visitor, func_decl);
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// fn walk_func_decl<V: Visitor>(visitor: &mut V, func_decl: &FuncDecl) {
//     for stmt in &func_decl.statements {
//         if visitor.visit_stmt(stmt).is_continue() {
//             match stmt {
//                 Stmt::Assign { local, expr } => {
//                     if visitor.visit_stmt_assign(*local, expr).is_continue() {
//                         walk_expr(visitor, expr);
//                     }
//                 }
//                 Stmt::Return(expr) => {
//                     if visitor.visit_stmt_return(expr).is_continue() {
//                         walk_expr(visitor, expr);
//                     }
//                 }
//             }
//         }
//     }
// }

// fn walk_expr<V: Visitor>(visitor: &mut V, expr: &Expr) {
//     visitor.visit_expr(expr);
//     match &expr.kind {
//         ExprKind::Constant(_) => {}
//         ExprKind::Var(_) => {}
//         ExprKind::UnOp { expr, .. } => visitor.visit_expr(expr),
//         ExprKind::BinOp { lhs, rhs, .. } => {
//             visitor.visit_expr(lhs);
//             visitor.visit_expr(rhs);
//         }
//     }
// }
