use super::ops::{BinOp, CmpOp, UnOp};
use crate::backend::registers::Register;
use crate::session::InternedStr;
use crate::utils::{declare_key_type, index_types};

index_types! { FuncId, BlockId }

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
    pub ops: Vec<Op>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Copy {
        source: Operand,
        dest: PlaceId,
    },

    UnOp {
        op: UnOp,
        source: Operand,
        dest: PlaceId,
    },

    BinOp {
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
        dest: PlaceId,
    },

    Call {
        func: FuncId,

        args: Vec<Operand>,
        dests: Vec<PlaceId>,
    },
}

impl Op {
    pub fn dests(&self) -> &[PlaceId] {
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
    Var(PlaceId),
}

declare_key_type! { pub struct PlaceId; }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Place {
    Register(Register),
    StackSlot(usize),
    Unresolved,
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
