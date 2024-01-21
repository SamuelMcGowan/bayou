use std::cell::{Ref, RefCell};
use std::fmt;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

#[derive(Default)]
pub struct Session {
    diagnostics: RefCell<Diagnostics>,
    interner: RefCell<Interner>,
}

impl Session {
    pub fn report(&self, diagnostic: impl IntoDiagnostic) {
        let diagnostic = diagnostic.into_diagnostic();
        self.diagnostics.borrow_mut().diagnostics.push(diagnostic);
    }

    pub fn intern(&self, s: impl AsRef<str>) -> InternedStr {
        self.interner.borrow_mut().get_or_intern(s)
    }

    /// Note: make sure string is dropped before calling [`Session::intern`] again.
    pub fn lookup_str(&self, s: InternedStr) -> Ref<str> {
        Ref::map(self.interner.borrow(), |i| i.resolve(&s))
    }

    pub fn into_inner(self) -> (Diagnostics, Interner) {
        (self.diagnostics.into_inner(), self.interner.into_inner())
    }
}

#[derive(Default, Debug, Clone)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn had_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for diagnostic in &self.diagnostics {
            writeln!(f, "error {}: {}", diagnostic.context, diagnostic.message)?;
        }
        Ok(())
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
