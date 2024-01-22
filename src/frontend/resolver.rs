/*
- resolve virtual registers
- allocate physical registers / stack slots
*/

use crate::ir::ast::{Item, Module};
use crate::ir::registers::{Ownership, VirtualRegister};
use crate::session::{Diagnostic, InternedStr, Session};
use crate::symbols::GlobalSymbol;

pub struct Resolver<'sess> {
    session: &'sess Session,
    locals: Locals,
}

impl<'sess> Resolver<'sess> {
    pub fn new(session: &'sess Session) -> Self {
        Self {
            session,
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

    fn declare_global(&mut self, name: InternedStr, symbol: GlobalSymbol) {
        let mut symbols = self.session.symbols.borrow_mut();

        if symbols.globals.insert(name, symbol).is_some() {
            let interner = self.session.interner.borrow();
            let name_str = interner.resolve(&name);

            self.session.diagnostics.report(Diagnostic::new(
                format!("duplicate global `{name_str}`"),
                "while resolving names",
            ));
        }
    }
}

#[derive(Default)]
struct Locals {
    vregs: Vec<Option<InternedStr>>,
    num_vregs: usize,

    scope_items: Vec<(VirtualRegister, Ownership)>,
}

impl Locals {
    pub fn reset(&mut self) {
        self.vregs.clear();
        self.num_vregs = 0;

        self.scope_items.clear();
    }

    pub fn push_owned(&mut self, name: Option<InternedStr>) -> VirtualRegister {
        if self.vregs.len() == isize::MAX as usize {
            panic!("don't know how you used that many registers");
        }

        let vreg = VirtualRegister(self.vregs.len());
        self.vregs.push(name);
        self.num_vregs = self.num_vregs.max(self.vregs.len());

        self.scope_items.push((vreg, Ownership::Owned));

        vreg
    }

    pub fn push_borrowed(&mut self, vreg: VirtualRegister) {
        self.scope_items.push((vreg, Ownership::Borrowed));
    }

    pub fn resolve_name(&self, name: InternedStr) -> Option<VirtualRegister> {
        for (i, name_candidate) in self.vregs.iter().enumerate().rev() {
            if *name_candidate == Some(name) {
                return Some(VirtualRegister(i));
            }
        }
        None
    }

    pub fn pop(&mut self) {
        let (_, ownership) = self.scope_items.pop().unwrap();
        if ownership == Ownership::Owned {
            self.vregs.pop();
        }
    }
}
