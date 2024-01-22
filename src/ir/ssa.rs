use super::ops::{BinOp, CmpOp, UnOp};
use crate::session::InternedStr;

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
pub struct ModuleIr {
    pub functions: Vec<FuncIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncIr {
    pub name: InternedStr,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub stmts: Vec<Op>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
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

impl Op {
    pub fn dests(&self) -> &[Var] {
        use std::slice::from_ref;

        match self {
            Op::Copy { source, dest } => from_ref(dest),
            Op::UnOp { op, source, dest } => from_ref(dest),
            Op::BinOp { op, lhs, rhs, dest } => from_ref(dest),
            Op::Call { func, args, dests } => dests,
        }
    }
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
