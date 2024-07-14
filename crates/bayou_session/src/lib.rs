pub mod diagnostics;
pub mod module_loader;
pub mod sourcemap;

use std::{collections::HashMap, path::PathBuf};

use bayou_interner::Interner;
use diagnostics::DiagnosticEmitter;
use diagnostics::*;
use module_loader::{FsLoader, HashMapLoader, ModuleLoader};
use sourcemap::SourceMap;
use target_lexicon::Triple;

pub struct ErrorsEmitted;

/// Session shared between multiple package compilations.
pub struct Session<D: DiagnosticEmitter> {
    pub target: Triple,
    pub sources: SourceMap,
    pub diagnostics: D,
}

impl<D: DiagnosticEmitter> Session<D> {
    pub fn new(target: Triple, diagnostics: D) -> Self {
        Self {
            target,
            sources: SourceMap::default(),
            diagnostics,
        }
    }

    pub fn report<Context>(
        &mut self,
        diagnostic: impl IntoDiagnostic<Context>,
        context: &Context,
    ) -> Result<(), ErrorsEmitted> {
        let diagnostic = diagnostic.into_diagnostic(context);
        let kind = diagnostic.severity;

        self.diagnostics.emit_diagnostic(diagnostic, &self.sources);

        if kind < Severity::Error {
            Ok(())
        } else {
            Err(ErrorsEmitted)
        }
    }

    pub fn report_all<Context, I>(
        &mut self,
        diagnostics: I,
        context: &Context,
    ) -> Result<(), ErrorsEmitted>
    where
        I: IntoIterator,
        I::Item: IntoDiagnostic<Context>,
    {
        let mut had_error = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(context);
            had_error |= diagnostic.severity >= Severity::Error;
            self.diagnostics.emit_diagnostic(diagnostic, &self.sources);
        }

        if !had_error {
            Ok(())
        } else {
            Err(ErrorsEmitted)
        }
    }
}

pub trait SessionTrait {
    type ModuleLoader: ModuleLoader;
    type PackageConfig;

    fn build_package_session(&self, config: Self::PackageConfig) -> PackageSession<Self>;

    fn target_triple(&self) -> &Triple;
    fn emit_diagnostic(&mut self, diagnostic: Diagnostic);

    fn report<Context>(
        &mut self,
        diagnostic: impl IntoDiagnostic<Context>,
        context: &Context,
    ) -> Result<(), ErrorsEmitted> {
        let diagnostic = diagnostic.into_diagnostic(context);
        let kind = diagnostic.severity;

        self.emit_diagnostic(diagnostic);

        if kind < Severity::Error {
            Ok(())
        } else {
            Err(ErrorsEmitted)
        }
    }

    fn report_all<Context, I>(
        &mut self,
        diagnostics: I,
        context: &Context,
    ) -> Result<(), ErrorsEmitted>
    where
        I: IntoIterator,
        I::Item: IntoDiagnostic<Context>,
    {
        let mut had_error = false;

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.into_diagnostic(context);
            had_error |= diagnostic.severity >= Severity::Error;
            self.emit_diagnostic(diagnostic);
        }

        if !had_error {
            Ok(())
        } else {
            Err(ErrorsEmitted)
        }
    }
}

pub struct PackageSession<S: SessionTrait + ?Sized> {
    pub interner: Interner,
    pub module_loader: S::ModuleLoader,
}

pub struct TestSession {
    pub target_triple: Triple,
    pub diagnostics: Vec<Diagnostic>,
}

impl SessionTrait for TestSession {
    type ModuleLoader = HashMapLoader;
    type PackageConfig = TestSessionConfig;

    fn build_package_session(&self, config: Self::PackageConfig) -> PackageSession<Self> {
        PackageSession {
            interner: Interner::new(),
            module_loader: HashMapLoader {
                modules: config.modules,
            },
        }
    }

    fn target_triple(&self) -> &Triple {
        &self.target_triple
    }

    fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

pub struct TestSessionConfig {
    pub modules: HashMap<String, String>,
}

pub struct FullSession {
    pub target_triple: Triple,

    pub diagnostics: PrettyDiagnosticEmitter,
    pub source_map: SourceMap,
}

impl SessionTrait for FullSession {
    type ModuleLoader = FsLoader;
    type PackageConfig = FullSessionConfig;

    fn build_package_session(&self, config: Self::PackageConfig) -> PackageSession<Self> {
        PackageSession {
            interner: Interner::new(),
            module_loader: FsLoader {
                root_dir: config.root_dir,
            },
        }
    }

    fn target_triple(&self) -> &Triple {
        &self.target_triple
    }

    fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics
            .emit_diagnostic(diagnostic, &self.source_map);
    }
}

pub struct FullSessionConfig {
    pub root_dir: PathBuf,
}
