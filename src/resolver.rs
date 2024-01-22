/*
- resolve virtual registers
- allocate physical registers / stack slots
*/

use crate::ast::registers::{Ownership, VirtualRegister};
use crate::session::InternedStr;

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
