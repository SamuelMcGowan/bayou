use std::{collections::HashMap, fs, io, path::PathBuf};

use bayou_interner::{Interner, Istr};
use bayou_session::{
    diagnostics::sources::Source as _,
    sourcemap::{Source, SourceId, SourceMap},
};

use crate::{
    ast,
    lexer::Lexer,
    module_tree::{DuplicateGlobalError, ModuleId, ModulePath, ModuleTree},
    parser::Parser,
    LexerError, ParseError,
};

pub enum GatherModulesError<M: ModuleLoader> {
    ModuleLoaderError(M::Error),

    LexerError(LexerError),
    ParseError(ParseError),

    InvalidModuleName(Istr),
    DuplicateGlobal(DuplicateGlobalError),
}

pub struct ParsedModule {
    pub scope_id: ModuleId,
    pub source_id: SourceId,

    pub ast: ast::Module,
}

pub struct ModuleGatherer<'a, 'src, M: ModuleLoader> {
    source_map: &'src mut SourceMap,
    module_loader: &'a M,

    errors: Vec<GatherModulesError<M>>,

    interner: &'a Interner,
}

impl<'a, 'src, M: ModuleLoader> ModuleGatherer<'a, 'src, M> {
    pub fn new(
        source_map: &'src mut SourceMap,
        module_loader: &'a M,
        interner: &'a Interner,
    ) -> Self {
        Self {
            source_map,
            module_loader,

            errors: vec![],

            interner,
        }
    }

    pub fn run(mut self) -> (ModuleTree, Vec<ParsedModule>, Vec<GatherModulesError<M>>) {
        let mut global_scope_tree = ModuleTree::new();
        let mut parsed_modules = vec![];

        let mut modules_to_load = vec![global_scope_tree.root_id()];

        while let Some(scope_id) = modules_to_load.pop() {
            let module_path = &global_scope_tree.entry(scope_id).path;

            let Some((source_id, ast)) = self.parse_module(module_path) else {
                continue;
            };

            parsed_modules.push(ParsedModule {
                scope_id,
                source_id,
                ast,
            });

            // TODO: use module submodules
            let submodule_names = vec![];

            for submodule_name in submodule_names {
                // A submodule of the root module can't be called `main` because it would
                // clash with the root module. For now just check no modules are called `main`.
                // TODO: handle cyclic modules properly?
                if &self.interner[submodule_name] == "main" {
                    self.errors
                        .push(GatherModulesError::InvalidModuleName(submodule_name));
                    continue;
                }

                let submodule_id = match global_scope_tree.insert_module(scope_id, submodule_name) {
                    Ok(id) => id,
                    Err(err) => {
                        self.errors.push(GatherModulesError::DuplicateGlobal(err));
                        continue;
                    }
                };

                modules_to_load.push(submodule_id);
            }
        }

        (global_scope_tree, parsed_modules, self.errors)
    }

    fn parse_module(&mut self, module_path: &ModulePath) -> Option<(SourceId, ast::Module)> {
        let source_string = match self.module_loader.load(module_path) {
            Ok(s) => s,
            Err(err) => {
                self.errors.push(GatherModulesError::ModuleLoaderError(err));
                return None;
            }
        };

        let (source_id, source) = self.source_map.insert_and_get(Source {
            name: module_path.display(self.interner).to_string(),
            source: source_string,
        });

        let (tokens, lexer_errors) = Lexer::new(source.source_str(), self.interner).lex();
        self.errors
            .extend(lexer_errors.into_iter().map(GatherModulesError::LexerError));

        let (ast, parse_errors) = Parser::new(tokens).parse();
        self.errors
            .extend(parse_errors.into_iter().map(GatherModulesError::ParseError));

        Some((source_id, ast))
    }
}

pub trait ModuleLoader {
    type Error;

    /// # Panics
    /// May panic or produce a wrong result if any of the path components are
    /// from a different interner than the this loader expects.
    fn load(&self, path: &ModulePath) -> Result<String, Self::Error>;
}

pub struct HashMapLoader(pub HashMap<ModulePath, String>);

impl ModuleLoader for HashMapLoader {
    type Error = ();

    fn load(&self, path: &ModulePath) -> Result<String, Self::Error> {
        self.0.get(path).cloned().ok_or(())
    }
}

pub struct FsLoader<'a> {
    pub root_dir: PathBuf,
    pub interner: &'a Interner,
}

impl ModuleLoader for FsLoader<'_> {
    type Error = io::Error;

    fn load(&self, path: &ModulePath) -> Result<String, Self::Error> {
        let path = module_path_to_pathbuf(path, &self.root_dir, self.interner);
        fs::read_to_string(path)
    }
}

fn module_path_to_pathbuf(
    module_path: &ModulePath,
    root_dir: impl Into<PathBuf>,
    interner: &Interner,
) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    match module_path.components() {
        [] => path.push("main.by"),

        [parents @ .., name] => {
            for &parent in parents {
                path.push(&interner[parent]);
            }

            path.push(&interner[*name]);
            path.set_extension("by");
        }
    }

    path
}
