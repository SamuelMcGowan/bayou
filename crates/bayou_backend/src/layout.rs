use bayou_ir::ir::Constant;
use bayou_ir::Type as IrType;
use cranelift::codegen::ir::{types, Type};

pub enum TypeLayout {
    Integer(Type),
    Void,
    Never,
}

impl TypeLayout {
    pub fn size(&self) -> usize {
        match self {
            Self::Integer(ty) => ty.bytes() as usize,
            Self::Void | Self::Never => 0,
        }
    }

    pub fn align(&self) -> usize {
        match self {
            Self::Integer(ty) => ty.bytes() as usize,
            Self::Void | Self::Never => 1,
        }
    }
}

pub trait TypeExt {
    fn layout(&self) -> TypeLayout;
}

impl TypeExt for IrType {
    fn layout(&self) -> TypeLayout {
        match self {
            Self::I64 => TypeLayout::Integer(types::I64),
            Self::Bool => TypeLayout::Integer(types::I8),
            Self::Void => TypeLayout::Void,
            Self::Never => TypeLayout::Never,
        }
    }
}

pub trait ConstantAsImm {
    /// Get this constant as an immediate value.
    ///
    /// Should be valid for its type.
    fn as_imm(&self) -> Option<i64>;
}

impl ConstantAsImm for Constant {
    fn as_imm(&self) -> Option<i64> {
        match self {
            Constant::I64(n) => Some(*n),
            Constant::Bool(b) => Some(if *b { 1 } else { 0 }),
            Constant::Void => None,
        }
    }
}
