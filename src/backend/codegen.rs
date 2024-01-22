use crate::ir::ssa::*;
use crate::session::Session;
use crate::symbols::SymbolTable;

pub struct CodeGenerator<'sess> {
    session: &'sess Session,
    symbols: SymbolTable,

    output: String,
}

impl<'sess> CodeGenerator<'sess> {
    pub fn new(session: &'sess Session, symbols: SymbolTable) -> Self {
        Self {
            session,
            symbols,

            output: String::new(),
        }
    }

    pub fn run(mut self, module: &ModuleIr) -> String {
        for func in &module.functions {
            self.gen_func(func);
        }

        self.output
    }

    fn gen_func(&mut self, func: &FuncIr) {
        let name = self.session.lookup_str(func.name);

        self.push_line(0, format!(".globl {name}"));
        self.push_line(0, format!("{name}:"));

        // FIXME: need to support multiple blocks!
        let block = &func.blocks[0];

        // FIXME: emit code for ops

        match &block.terminator {
            Terminator::Jump { .. } => todo!(),
            Terminator::JumpIf { .. } => todo!(),
            Terminator::Return(operands) => {
                // FIXME: support more than one return value
                match operands[0] {
                    Operand::Constant(n) => {
                        self.push_line(1, format!("movq ${n}, %rax"));
                        self.push_line(1, "ret");
                    }
                    // FIXME: support variables properly
                    Operand::Var(_) => {}
                }
            }
        }
    }

    fn push_line(&mut self, indent: u8, s: impl AsRef<str>) {
        const INDENT: &str = "    ";

        for _ in 0..indent {
            self.output.push_str(INDENT);
        }

        self.output.push_str(s.as_ref());
        self.output.push('\n');
    }
}
