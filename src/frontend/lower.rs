use super::ast::*;
use crate::ir::ssa::*;

pub fn lower(module: Module) -> ModuleIr {
    match module.item {
        Item::FuncDecl(func_decl) => {
            let func_ir = lower_func_decl(func_decl);
            ModuleIr {
                functions: vec![func_ir],
            }
        }
        Item::ParseError => unreachable!(),
    }
}

fn lower_func_decl(func: FuncDecl) -> FuncIr {
    let block = match func.statement {
        Stmt::Return(expr) => {
            let mut vars = Vars::default();

            let expr_ir = lower_expr(expr, &mut vars);

            let expr_outputs = expr_ir
                .dests()
                .iter()
                .map(|&var| Operand::Var(var))
                .collect();

            BasicBlock {
                stmts: vec![],
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

fn lower_expr(expr: Expr, vars: &mut Vars) -> Op {
    match expr {
        Expr::Constant(n) => {
            let var = vars.create_var();
            Op::Copy {
                source: Operand::Constant(n),
                dest: var,
            }
        }
    }
}

#[derive(Default)]
struct Vars(usize);

impl Vars {
    pub fn create_var(&mut self) -> Var {
        let var = Var(self.0);
        self.0 += 1;
        var
    }
}
