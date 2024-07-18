use bayou_session::sourcemap::SourceSpan;
use bayou_utils::keyvec::{declare_key_type, KeyVec};

use crate::{IdentWithSource, Type};

#[derive(Default, Debug, Clone)]
pub struct Symbols {
    pub locals: KeyVec<LocalId, LocalSymbol>,
    pub funcs: KeyVec<FuncId, FunctionSymbol>,
}

declare_key_type! { pub struct LocalId; }
declare_key_type! { pub struct FuncId; }

#[derive(Debug, Clone)]
pub struct LocalSymbol {
    pub ident: IdentWithSource,

    pub ty: Type,
    pub ty_span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub ident: IdentWithSource,

    pub ret_ty: Type,
    pub ret_ty_span: SourceSpan,
}
