use std::collections::HashMap;

use bayou_diagnostic::span::Span;

use crate::ir::ir::Type;
use crate::ir::{Ident, InternedStr};
use crate::utils::keyvec::{declare_key_type, KeyVec};

#[derive(Default, Debug, Clone)]
pub struct Symbols {
    pub globals: HashMap<InternedStr, GlobalSymbol>,
    pub locals: KeyVec<LocalId, LocalSymbol>,
}

declare_key_type! { pub struct LocalId; }

#[derive(Debug, Clone)]
pub struct LocalSymbol {
    pub ident: Ident,

    pub ty: Type,
    pub ty_span: Span,
}

#[derive(Debug, Clone)]
pub struct GlobalSymbol {
    pub ident: Ident,
}
