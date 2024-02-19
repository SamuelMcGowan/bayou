use std::collections::HashMap;

use bayou_diagnostic::span::Span;

use crate::ir::ir::Type;
use crate::ir::{Ident, InternedStr};
use crate::utils::keyvec::{declare_key_type, KeyVec};

#[derive(Default, Debug, Clone)]
pub struct Symbols {
    pub global_lookup: HashMap<InternedStr, GlobalId>,
    pub locals: KeyVec<LocalId, LocalSymbol>,

    pub funcs: KeyVec<FuncId, FunctionSymbol>,
}

impl Symbols {
    pub fn get_global_ident(&self, id: GlobalId) -> Option<Ident> {
        match id {
            GlobalId::Func(id) => self.funcs.get(id).map(|s| s.ident),
        }
    }
}

declare_key_type! { pub struct LocalId; }
declare_key_type! { pub struct FuncId; }

#[derive(Debug, Clone)]
pub struct LocalSymbol {
    pub ident: Ident,

    pub ty: Type,
    pub ty_span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum GlobalId {
    Func(FuncId),
}

#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub ident: Ident,
    pub ret_ty: Type,
}
