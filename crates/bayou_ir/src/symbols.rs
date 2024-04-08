use std::collections::HashMap;

use bayou_diagnostic::span::Span;
use bayou_utils::keyvec::{declare_key_type, KeyVec};

use crate::{Ident, InternedStr, Type};

#[derive(Default, Debug, Clone)]
pub struct Symbols {
    pub global_lookup: HashMap<InternedStr, GlobalId>,
    pub locals: KeyVec<LocalId, LocalSymbol>,

    pub funcs: KeyVec<FuncId, FunctionSymbol>,
}

impl Symbols {
    pub fn lookup_global(&self, name: InternedStr) -> Option<GlobalId> {
        self.global_lookup.get(&name).copied()
    }

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

impl GlobalId {
    pub fn as_func(self) -> Option<FuncId> {
        match self {
            Self::Func(id) => Some(id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub ident: Ident,
    pub ret_ty: Type,
}
