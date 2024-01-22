use super::ops::{BinOp, CmpOp, UnOp};

macro_rules! index_types {
    ($($t:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $t(pub usize);
        )*
    };
}

index_types! { FuncId, BlockId, Var }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub stmts: Vec<Statement>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Copy {
        source: Operand,
        dest: Var,
    },

    UnOp {
        op: UnOp,
        source: Operand,
        dest: Var,
    },

    BinOp {
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
        dest: Var,
    },

    Call {
        func: FuncId,

        args: Vec<Operand>,
        dests: Vec<Var>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operand {
    Constant(u64),
    Var(Var),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    Jump {
        dest_block: BlockId,
        args: Vec<Operand>,
    },

    JumpIf {
        lhs: Operand,
        rhs: Operand,
        op: CmpOp,

        dest_block: BlockId,
        args: Vec<Operand>,
    },

    Return(Vec<Operand>),
}
