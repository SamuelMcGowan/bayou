use super::registers::Register;
use crate::ir::ssa::*;
use crate::session::Symbols;

struct Frame {
    regs: [bool; 16],
    stack: Vec<bool>,
}

impl Default for Frame {
    fn default() -> Self {
        let mut frame = Frame {
            regs: [false; 16],
            stack: vec![],
        };

        frame.reserve_register(Register::Rsp);
        frame.reserve_register(Register::Rax);

        frame
    }
}

impl Frame {
    /// Mark a register as in-use.
    ///
    /// Panics if the register is already in use.
    pub fn reserve_register(&mut self, reg: Register) {
        if self.regs[reg as usize] {
            panic!("register {reg:?} already in use");
        }
    }

    /// Allocate a location.
    ///
    /// Tries to allocate a register first, if one is not available,
    /// spills to the stack.
    pub fn alloc(&mut self) -> Place {
        if let Some(reg) = self.alloc_reg() {
            Place::Register(reg)
        } else {
            Place::StackSlot(self.alloc_stack())
        }
    }

    /// Free a location, marking it for reuse.
    ///
    /// Panics if the location is already free, is unresolved, or
    /// (if it's a stack offset) it's out of bounds.
    pub fn free(&mut self, place: Place) {
        match place {
            Place::Register(reg) => self.free_reg(reg),
            Place::StackSlot(slot) => self.free_stack(slot),
            Place::Unresolved => {}
        }
    }

    fn alloc_reg(&mut self) -> Option<Register> {
        self.regs
            .iter()
            .position(|&in_use| !in_use)
            .and_then(|i| Register::n(i as u8))
    }

    /// Panics if the register is already free.
    fn free_reg(&mut self, reg: Register) {
        if !self.regs[reg as usize] {
            panic!("register {reg:?} already free");
        }

        self.regs[reg as usize] = false;
    }

    fn alloc_stack(&mut self) -> usize {
        let reuse_slot = self.stack.iter().position(|&in_use| !in_use);

        match reuse_slot {
            Some(slot) => {
                self.stack[slot] = true;
                slot
            }
            None => {
                let slot = self.stack.len();
                self.stack.push(true);
                slot
            }
        }
    }

    /// Panics if the slot is already free or is out of bounds.
    fn free_stack(&mut self, slot: usize) {
        if !self.regs[slot] {
            panic!("stack slot {slot} already free");
        }
    }
}

pub fn regalloc(mut symbols: Symbols, module: &ModuleIr) -> Symbols {
    for func in &module.functions {
        let mut frame = Frame::default();

        let mut declare_place = |place_id| {
            let place = frame.alloc();
            symbols.places[place_id] = place;
        };

        // TODO: free places once they are dead

        for block in &func.blocks {
            for op in &block.ops {
                match op {
                    Op::Copy { dest, .. } => declare_place(*dest),
                    Op::UnOp { dest, .. } => declare_place(*dest),
                    Op::BinOp { dest, .. } => declare_place(*dest),
                    Op::Call { dests, .. } => {
                        for dest in dests {
                            declare_place(*dest);
                        }
                    }
                }
            }
        }
    }

    symbols
}
