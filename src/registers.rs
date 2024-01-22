use std::fmt;

pub enum Location {
    Reg(Register),
    StackOffset(usize),
}

#[derive(enumn::N, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rbx = 3,
    Rsi = 4,
    Rdi = 5,
    Rsp = 6,
    Rbp = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Register::Rax => "rax",
            Register::Rcx => "rcx",
            Register::Rdx => "rdx",
            Register::Rbx => "rbx",
            Register::Rsi => "rsi",
            Register::Rdi => "rdi",
            Register::Rsp => "rsp",
            Register::Rbp => "rbp",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
        };

        write!(f, "{name}")
    }
}

pub type ScopeState = usize;

pub struct RegAlloc {
    regs: [bool; 16],
}

impl RegAlloc {
    pub fn new() -> Self {
        let mut alloc = Self { regs: [false; 16] };

        alloc.reserve(Register::Rsp);

        alloc
    }

    /// Mark a register as in-use.
    ///
    /// Panics if the register is already in use.
    pub fn reserve(&mut self, reg: Register) {
        if self.regs[reg as usize] {
            panic!("register {reg:?} already in use");
        }

        self.regs[reg as usize] = true;
    }

    pub fn alloc(&mut self) -> Option<Register> {
        self.regs
            .iter()
            .position(|&in_use| !in_use)
            .and_then(|i| Register::n(i as u8))
    }

    /// Panics if the register is already free.
    pub fn free(&mut self, reg: Register) {
        if !self.regs[reg as usize] {
            panic!("register {reg:?} already free");
        }

        self.regs[reg as usize] = false;
    }

    /// Get all callee-saved registers.
    pub fn callee_saved(&self) -> &[Register] {
        &[
            Register::Rbp,
            Register::R12,
            Register::R13,
            Register::R14,
            Register::R15,
        ]
    }

    /// Get all (currently allocated) caller-saved registers.
    pub fn caller_saved(&self) -> impl Iterator<Item = Register> + '_ {
        const CALLER_SAVED: &[Register] = &[
            Register::Rax,
            Register::Rcx,
            Register::Rdx,
            Register::Rbx,
            Register::Rsi,
            Register::Rdi,
            Register::Rsp,
            Register::R8,
            Register::R9,
            Register::R10,
            Register::R11,
        ];

        CALLER_SAVED
            .iter()
            .copied()
            .filter(|&reg| !self.regs[reg as usize])
    }
}
