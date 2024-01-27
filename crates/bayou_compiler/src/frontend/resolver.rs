use bayou_diagnostic::{Diagnostic, Snippet};

use crate::compiler::ModuleContext;
use crate::ir::ast::{Ident, Item, Module};
use crate::ir::vars::{Ownership, Place, PlaceRef};
use crate::ir::InternedStr;
use crate::symbols::GlobalSymbol;

pub struct Resolver<'cx> {
    context: &'cx mut ModuleContext,
    locals: Locals,
}

impl<'cx> Resolver<'cx> {
    pub fn new(context: &'cx mut ModuleContext) -> Self {
        Self {
            context,
            locals: Locals::default(),
        }
    }

    pub fn run(mut self, module: &mut Module) {
        self.declare_globals(std::slice::from_mut(&mut module.item));
    }

    fn declare_globals(&mut self, items: &mut [Item]) {
        for item in items.iter_mut() {
            match item {
                Item::FuncDecl(func) => self.declare_global(func.name, GlobalSymbol {}),
                Item::ParseError => unreachable!(),
            }
        }
    }

    fn declare_global(&mut self, ident: Ident, symbol: GlobalSymbol) {
        if self
            .context
            .symbols
            .globals
            .insert(ident.ident, symbol)
            .is_some()
        {
            let name_str = self.context.interner.resolve(&ident.ident);

            self.context.diagnostics.report(
                Diagnostic::error()
                    .with_message(format!("duplicate global `{name_str}`"))
                    .with_snippet(Snippet::primary(
                        "duplicate global",
                        self.context.source_id,
                        ident.span,
                    )),
            );
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
