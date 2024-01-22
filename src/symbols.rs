use std::marker::PhantomData;
use std::{fmt, hash};

use crate::backend::registers::Register;
use crate::session::InternedStr;

pub struct SymbolId<T> {
    id: usize,
    _phantom: PhantomData<*const T>,
}

impl<T> fmt::Debug for SymbolId<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("SymbolId").field(&self.id).finish()
    }
}

impl<T> Clone for SymbolId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SymbolId<T> {}

impl<T> PartialEq for SymbolId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for SymbolId<T> {}

impl<T> hash::Hash for SymbolId<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Default, Debug, Clone)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn insert<S: SymbolKind>(&mut self, symbol: S) -> SymbolId<S> {
        let id = SymbolId {
            id: self.symbols.len(),
            _phantom: PhantomData,
        };
        self.symbols.push(symbol.into_symbol());
        id
    }

    pub fn get<S: SymbolKind>(&self, id: SymbolId<S>) -> Option<&S> {
        self.symbols.get(id.id).and_then(S::from_symbol)
    }

    pub fn get_mut<S: SymbolKind>(&mut self, id: SymbolId<S>) -> Option<&mut S> {
        self.symbols.get_mut(id.id).and_then(S::from_symbol_mut)
    }
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Var(VarSymbol),
}

pub trait SymbolKind {
    fn into_symbol(self) -> Symbol;
    fn from_symbol(symbol: &Symbol) -> Option<&Self>;
    fn from_symbol_mut(symbol: &mut Symbol) -> Option<&mut Self>;
}

#[derive(Default, Debug, Clone)]
pub struct VarSymbol {
    pub name: Option<InternedStr>,
    pub register: Option<Register>,
}

impl SymbolKind for VarSymbol {
    fn into_symbol(self) -> Symbol {
        Symbol::Var(self)
    }

    fn from_symbol(symbol: &Symbol) -> Option<&Self> {
        match symbol {
            Symbol::Var(var) => Some(var),
        }
    }

    fn from_symbol_mut(symbol: &mut Symbol) -> Option<&mut Self> {
        match symbol {
            Symbol::Var(var) => Some(var),
        }
    }
}
