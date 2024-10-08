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

#[derive(thiserror::Error, Debug, Clone, Copy)]
#[error("errors emitted")]
pub struct ErrorsEmitted;

/// Session shared between multiple package compilations.
pub trait Session {
    type ModuleLoader: ModuleLoader;
    type PackageConfig;

    fn build_package_session(&self, config: Self::PackageConfig) -> PackageSession<Self>;

    fn source_map(&self) -> &SourceMap;
    fn source_map_mut(&mut self) -> &mut SourceMap;

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

        if had_error {
            Err(ErrorsEmitted)
        } else {
            Ok(())
        }
    }
}

/// Like [`Session`] but supporting code generation.
pub trait CodegenSession: Session {
    fn target_triple(&self) -> &Triple;
}

/// Session for a single package compilation.
#[derive(Debug)]
pub struct PackageSession<S: Session + ?Sized> {
    pub name: String,
    pub interner: Interner,
    pub module_loader: S::ModuleLoader,
}

#[derive(Default, Debug, Clone)]
pub struct TestSession {
    pub diagnostics: Vec<Diagnostic>,
    pub source_map: SourceMap,
}

impl TestSession {
    pub fn new() -> Self {
        Self {
            diagnostics: vec![],
            source_map: SourceMap::default(),
        }
    }
}

impl Session for TestSession {
    type ModuleLoader = HashMapLoader;
    type PackageConfig = TestSessionConfig;

    fn build_package_session(
        &self,
        TestSessionConfig { name, modules }: Self::PackageConfig,
    ) -> PackageSession<Self> {
        PackageSession {
            name,
            interner: Interner::new(),
            module_loader: HashMapLoader { modules },
        }
    }

    fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    fn source_map_mut(&mut self) -> &mut SourceMap {
        &mut self.source_map
    }

    fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

#[derive(Debug, Clone)]
pub struct TestSessionConfig {
    pub name: String,
    pub modules: HashMap<String, String>,
}

impl TestSessionConfig {
    pub fn new(name: impl Into<String>, modules: impl Into<HashMap<String, String>>) -> Self {
        Self {
            name: name.into(),
            modules: modules.into(),
        }
    }
}

#[derive(Debug)]
pub struct FullSession {
    pub target_triple: Triple,

    pub diagnostics: PrettyDiagnosticEmitter,
    pub source_map: SourceMap,
}

impl FullSession {
    pub fn new(target_triple: Triple) -> Self {
        Self {
            target_triple,
            diagnostics: PrettyDiagnosticEmitter::default(),
            source_map: SourceMap::default(),
        }
    }
}

impl Session for FullSession {
    type ModuleLoader = FsLoader;
    type PackageConfig = FullSessionConfig;

    fn build_package_session(
        &self,
        FullSessionConfig { name, root_dir }: Self::PackageConfig,
    ) -> PackageSession<Self> {
        PackageSession {
            name,
            interner: Interner::new(),
            module_loader: FsLoader { root_dir },
        }
    }

    fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    fn source_map_mut(&mut self) -> &mut SourceMap {
        &mut self.source_map
    }

    fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics
            .emit_diagnostic(diagnostic, &self.source_map);
    }
}

impl CodegenSession for FullSession {
    fn target_triple(&self) -> &Triple {
        &self.target_triple
    }
}

#[derive(Debug, Clone)]
pub struct FullSessionConfig {
    pub name: String,
    pub root_dir: PathBuf,
}
