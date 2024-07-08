use std::collections::HashMap;

use bayou_interner::Istr;
use bayou_session::sourcemap::SourceSpan;
use bayou_utils::keyvec::{declare_key_type, KeyVec};

use crate::{IdentWithSource, Type};

#[derive(Default, Debug, Clone)]
pub struct Symbols {
    // FIXME: globals should have already been resolved by this point
    pub global_lookup: HashMap<Istr, GlobalId>,
    pub locals: KeyVec<LocalId, LocalSymbol>,

    pub funcs: KeyVec<FuncId, FunctionSymbol>,
}

impl Symbols {
    pub fn lookup_global(&self, name: Istr) -> Option<GlobalId> {
        self.global_lookup.get(&name).copied()
    }

    pub fn get_global_ident(&self, id: GlobalId) -> Option<IdentWithSource> {
        match id {
            GlobalId::Func(id) => self.funcs.get(id).map(|s| s.ident),
        }
    }
}

declare_key_type! { pub struct LocalId; }
declare_key_type! { pub struct FuncId; }

#[derive(Debug, Clone)]
pub struct LocalSymbol {
    pub ident: IdentWithSource,

    pub ty: Type,
    pub ty_span: SourceSpan,
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
    pub ident: IdentWithSource,

    pub ret_ty: Type,
    pub ret_ty_span: SourceSpan,
}
