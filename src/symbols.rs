use std::collections::HashMap;

use crate::session::InternedStr;
use crate::utils::{declare_key_type, KeyVec};

// Nothing stored here at the moment, but I suspect it'd be a pain
// adding a symbol table later, so I'm wiring it up now.
#[derive(Default, Debug, Clone)]
pub struct Symbols {
    pub globals: HashMap<InternedStr, GlobalSymbol>,
    pub locals: KeyVec<LocalId, LocalSymbol>,
}

declare_key_type! { pub struct LocalId; }

#[derive(Debug, Clone)]
pub struct LocalSymbol {}

#[derive(Debug, Clone)]
pub struct GlobalSymbol {}
