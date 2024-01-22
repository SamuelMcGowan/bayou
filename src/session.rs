use std::cell::{Cell, RefCell};

use crate::symbols::Symbols;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

#[derive(Default)]
pub struct Session {
    pub diagnostics: Diagnostics,
    pub symbols: RefCell<Symbols>,
    pub interner: RefCell<Interner>,
}

#[derive(Default, Debug)]
pub struct Diagnostics {
    diagnostics: RefCell<Vec<Diagnostic>>,
    had_errors: Cell<bool>,
}

impl Diagnostics {
    pub fn report(&self, diagnostic: impl IntoDiagnostic) {
        let diagnostic = diagnostic.into_diagnostic();
        self.diagnostics.borrow_mut().push(diagnostic);
        self.had_errors.set(true);
    }

    pub fn had_errors(&self) -> bool {
        self.had_errors.get()
    }

    pub fn flush_diagnostics(&self) {
        for diagnostic in self.diagnostics.borrow_mut().drain(..) {
            eprintln!("error {}: {}", diagnostic.context, diagnostic.message);
        }
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
