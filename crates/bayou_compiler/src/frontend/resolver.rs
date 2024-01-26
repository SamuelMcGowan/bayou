use bayou_diagnostic::Diagnostic;

use crate::diagnostics::Diagnostics;
use crate::ir::ast::{Item, Module};
use crate::ir::vars::{Ownership, Place, PlaceRef};
use crate::ir::{InternedStr, Interner};
use crate::symbols::{GlobalSymbol, Symbols};

pub struct Resolver<'sess> {
    symbols: Symbols,
    interner: &'sess Interner,
    diagnostics: Diagnostics,

    locals: Locals,
}

impl<'sess> Resolver<'sess> {
    pub fn new(interner: &'sess Interner) -> Self {
        Self {
            symbols: Symbols::default(),
            interner,
            diagnostics: Diagnostics::default(),

            locals: Locals::default(),
        }
    }

    pub fn run(mut self, module: &mut Module) -> (Symbols, Diagnostics) {
        self.declare_globals(std::slice::from_mut(&mut module.item));
        (self.symbols, self.diagnostics)
    }

    fn declare_globals(&mut self, items: &mut [Item]) {
        for item in items.iter_mut() {
            match item {
                Item::FuncDecl(func) => self.declare_global(func.name, GlobalSymbol {}),
                Item::ParseError => unreachable!(),
            }
        }
    }

    fn declare_global(&mut self, name: InternedStr, symbol: GlobalSymbol) {
        if self.symbols.globals.insert(name, symbol).is_some() {
            let name_str = self.interner.resolve(&name);

            self.diagnostics
                .report(Diagnostic::error().with_message(format!("duplicate global `{name_str}`")));
        }
    }
}

#[derive(Default)]
struct Locals {
    places: Vec<Option<InternedStr>>,
    scope_items: Vec<PlaceRef>,
}

impl Locals {
    pub fn reset(&mut self) {
        self.places.clear();
        self.scope_items.clear();
    }

    pub fn push_owned(&mut self, name: Option<InternedStr>) -> Place {
        if self.places.len() == isize::MAX as usize {
            panic!("don't know how you used that many registers");
        }

        let place = Place(self.places.len());
        self.places.push(name);

        self.scope_items.push(PlaceRef::owned(place));

        place
    }

    pub fn push_borrowed(&mut self, place: Place) {
        self.scope_items.push(PlaceRef::borrowed(place));
    }

    pub fn resolve_name(&self, name: InternedStr) -> Option<Place> {
        for (i, name_candidate) in self.places.iter().enumerate().rev() {
            if *name_candidate == Some(name) {
                return Some(Place(i));
            }
        }
        None
    }

    pub fn pop(&mut self) {
        let place_ref = self.scope_items.pop().unwrap();
        if place_ref.ownership == Ownership::Owned {
            self.places.pop();
        }
    }
}
