use super::ast::*;
use crate::ir::ssa::*;
use crate::session::Symbols;

#[derive(Default)]
pub struct Lowerer {
    symbols: Symbols,
}

impl Lowerer {
    pub fn run(mut self, module: Module) -> (ModuleIr, Symbols) {
        match module.item {
            Item::FuncDecl(func_decl) => {
                let func_ir = self.lower_func_decl(func_decl);
                let mod_ir = ModuleIr {
                    functions: vec![func_ir],
                };
                (mod_ir, self.symbols)
            }
            Item::ParseError => unreachable!(),
        }
    }

    fn lower_func_decl(&mut self, func: FuncDecl) -> FuncIr {
        let block = match func.statement {
            Stmt::Return(expr) => {
                let expr_ir = self.lower_expr(expr);

                let expr_outputs = expr_ir
                    .dests()
                    .iter()
                    .map(|&var| Operand::Var(var))
                    .collect();

                BasicBlock {
                    ops: vec![],
                    terminator: Terminator::Return(expr_outputs),
                }
            }

            Stmt::ParseError => unreachable!(),
        };

        FuncIr {
            name: func.name,
            blocks: vec![block],
        }
    }

    fn lower_expr(&mut self, expr: Expr) -> Op {
        match expr {
            Expr::Constant(n) => Op::Copy {
                source: Operand::Constant(n),
                dest: self.symbols.places.insert(Place::Unresolved),
            },
        }
    }
}
