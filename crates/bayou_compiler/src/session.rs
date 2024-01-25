use std::cell::RefCell;

use crate::diagnostic::{DiagnosticOutput, IntoDiagnostic, Sources};
use crate::symbols::Symbols;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

#[derive(Debug)]
pub struct Session {
    pub diagnostics: RefCell<DiagnosticOutput>,
    pub sources: RefCell<Sources>,

    pub symbols: RefCell<Symbols>,
    pub interner: RefCell<Interner>,
}

impl Session {
    pub fn new(diagnostics: DiagnosticOutput) -> Self {
        Self {
            diagnostics: RefCell::new(diagnostics),
            sources: RefCell::default(),

            symbols: RefCell::default(),
            interner: RefCell::default(),
        }
    }

    pub fn report(&self, diagnostic: impl IntoDiagnostic) {
        self.diagnostics
            .borrow_mut()
            .report(diagnostic, self.sources.borrow().as_ref());
    }

    pub fn had_errors(&self) -> bool {
        self.diagnostics.borrow().had_errors()
    }
}
