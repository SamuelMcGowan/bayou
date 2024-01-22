use crate::ir::ssa::*;
use crate::session::{Session, Symbols};

pub struct CodeGenerator<'sess> {
    session: &'sess Session,
    symbols: Symbols,

    output: String,
}

impl<'sess> CodeGenerator<'sess> {
    pub fn new(session: &'sess Session, symbols: Symbols) -> Self {
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

        for op in &block.ops {
            match op {
                Op::Copy { source, dest } => {
                    self.push_line(
                        1,
                        format!(
                            "movq {}, {}",
                            self.operand_to_str(*source),
                            self.place_to_str(*dest)
                        ),
                    );
                }
                _ => todo!(),
            }
        }

        match &block.terminator {
            Terminator::Jump { .. } => todo!(),
            Terminator::JumpIf { .. } => todo!(),
            Terminator::Return(_) => {
                self.push_line(1, "ret");
            }
        }
    }

    fn operand_to_str(&self, operand: Operand) -> String {
        match operand {
            Operand::Constant(n) => format!("${n}"),
            Operand::Var(place) => self.place_to_str(place),
        }
    }

    fn place_to_str(&self, place: PlaceId) -> String {
        match self.symbols.places[place] {
            Place::Register(reg) => format!("%{reg}"),
            Place::StackSlot(slot) => format!("{}(rsp)", slot * 8),
            Place::Unresolved => unreachable!(),
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
