use std::cell::{Cell, Ref, RefCell};

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

#[derive(Default)]
pub struct Session {
    diagnostics: RefCell<Vec<Diagnostic>>,
    had_errors: Cell<bool>,

    interner: RefCell<Interner>,
}

impl Session {
    pub fn report(&self, diagnostic: impl IntoDiagnostic) {
        let diagnostic = diagnostic.into_diagnostic();
        self.diagnostics.borrow_mut().push(diagnostic);
        self.had_errors.set(true);
    }

    pub fn flush_diagnostics(&self) {
        for diagnostic in self.diagnostics.borrow().iter() {
            eprintln!("error {}: {}", diagnostic.context, diagnostic.message);
        }
    }

    pub fn had_errors(&self) -> bool {
        self.had_errors.get()
    }

    pub fn intern(&self, s: impl AsRef<str>) -> InternedStr {
        self.interner.borrow_mut().get_or_intern(s)
    }

    /// Note: make sure string is dropped before calling [`Session::intern`] again.
    pub fn lookup_str(&self, s: InternedStr) -> Ref<str> {
        Ref::map(self.interner.borrow(), |i| i.resolve(&s))
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub context: String,
}

pub trait IntoDiagnostic {
    fn into_diagnostic(self) -> Diagnostic;
}

use crate::ir::ssa::{Place, PlaceId};
use crate::utils::KeyVec;

#[derive(Default, Debug, Clone)]
pub struct Symbols {
    pub places: KeyVec<PlaceId, Place>,
}
