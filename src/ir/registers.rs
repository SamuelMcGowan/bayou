#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Place {
    /// A location that hasn't been allocated a physical location yet.
    VirtualRegister(VirtualRegister),

    /// A physical register.
    Register(Register),

    /// A slot on the stack.
    StackSlot(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ownership {
    Owned,
    Borrowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualRegister(pub usize);

#[derive(enumn::N, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Register {
    Rax,
    Rcx,
    Rdx,
    Rbx,
    Rsi,
    Rsp,
    Rbp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}
