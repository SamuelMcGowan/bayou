use bayou_interner::Interner;
use bayou_ir::IdentWithSource;
use bayou_session::{
    diagnostics::{prelude::*, sources::Source as _},
    module_loader::{ModuleLoader, ModulePath},
    sourcemap::{Source, SourceId},
    PackageSession, Session,
};

use crate::{
    ast,
    lexer::Lexer,
    module_tree::{DuplicateGlobalError, ModuleId, ModuleTree},
    parser::Parser,
    LexerError, ParseError,
};

pub enum GatherModulesError<S: Session> {
    ModuleLoaderError(<S::ModuleLoader as ModuleLoader>::Error),

    LexerError(LexerError, SourceId),
    ParseError(ParseError, SourceId),

    InvalidModuleName(IdentWithSource),
    DuplicateGlobal(DuplicateGlobalError),
}

impl<S: Session> IntoDiagnostic<Interner> for GatherModulesError<S> {
    fn into_diagnostic(self, interner: &Interner) -> bayou_session::diagnostics::Diagnostic {
        match self {
            Self::ModuleLoaderError(_) => {
                // TODO: require ModuleLoader::Error to be IntoDiagnostic
                todo!()
            }

            Self::LexerError(err, source_id) => err.into_diagnostic(&source_id),
            Self::ParseError(err, source_id) => err.into_diagnostic(&source_id),

            Self::InvalidModuleName(name) => Diagnostic::error()
                .with_message("invalid module name")
                .with_snippet(Snippet::primary(
                    format!(
                        "`{}` is not a valid name for a module",
                        interner.get_str(name.istr).unwrap()
                    ),
                    name.span.source_id,
                    name.span.span,
                )),

            Self::DuplicateGlobal(_err) => {
                // TODO: store source spans in duplicate global error
                todo!()
            }
        }
    }
}

pub struct ParsedModule {
    pub scope_id: ModuleId,
    pub source_id: SourceId,

    pub ast: ast::Module,
}

pub struct ModuleGatherer<'a, S: Session> {
    session: &'a mut S,
    package_session: &'a mut PackageSession<S>,

    errors: Vec<GatherModulesError<S>>,
}

impl<'a, S: Session> ModuleGatherer<'a, S> {
    pub fn new(session: &'a mut S, package_session: &'a mut PackageSession<S>) -> Self {
        Self {
            session,
            package_session,

            errors: vec![],
        }
    }

    pub fn run(mut self) -> (ModuleTree, Vec<ParsedModule>, Vec<GatherModulesError<S>>) {
        let mut global_scope_tree = ModuleTree::new();
        let mut parsed_modules = vec![];

        let mut modules_to_load = vec![global_scope_tree.root_id()];

        while let Some(scope_id) = modules_to_load.pop() {
            let module_path = &global_scope_tree.entry(scope_id).path;

            let Some((source_id, ast)) = self.parse_module(module_path) else {
                continue;
            };

            let submodule_names = ast.items.iter().filter_map(|item| match item {
                ast::Item::Submodule(name) => Some(name.with_source(source_id)),
                _ => None,
            });

            for submodule_name in submodule_names {
                // A submodule of the root module can't be called `main` because it would
                // clash with the root module. For now just check no modules are called `main`.
                // TODO: handle cyclic modules properly?
                if &self.package_session.interner[submodule_name.istr] == "main" {
                    // span is in
                    self.errors
                        .push(GatherModulesError::InvalidModuleName(submodule_name));
                    continue;
                }

                let submodule_id =
                    match global_scope_tree.insert_module(scope_id, submodule_name.istr) {
                        Ok(id) => id,
                        Err(err) => {
                            self.errors.push(GatherModulesError::DuplicateGlobal(err));
                            continue;
                        }
                    };

                modules_to_load.push(submodule_id);
            }

            parsed_modules.push(ParsedModule {
                scope_id,
                source_id,
                ast,
            });
        }

        (global_scope_tree, parsed_modules, self.errors)
    }

    fn parse_module(&mut self, module_path: &ModulePath) -> Option<(SourceId, ast::Module)> {
        let source_string = match self
            .package_session
            .module_loader
            .load_module(module_path, &self.package_session.interner)
        {
            Ok(s) => s,
            Err(err) => {
                self.errors.push(GatherModulesError::ModuleLoaderError(err));
                return None;
            }
        };

        let (source_id, source) = self.session.source_map_mut().insert_and_get(Source {
            name: module_path
                .display(&self.package_session.interner)
                .to_string(),
            source: source_string,
        });

        let (tokens, lexer_errors) =
            Lexer::new(source.source_str(), &self.package_session.interner).lex();
        self.errors.extend(
            lexer_errors
                .into_iter()
                .map(|err| GatherModulesError::LexerError(err, source_id)),
        );

        let (ast, parse_errors) = Parser::new(tokens).parse();
        self.errors.extend(
            parse_errors
                .into_iter()
                .map(|err| GatherModulesError::ParseError(err, source_id)),
        );

        Some((source_id, ast))
    }
}
