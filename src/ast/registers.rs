#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Place {
    pub kind: PlaceKind,
    pub owned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaceKind {
    Register(Register),
    Stack(usize),
}

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
