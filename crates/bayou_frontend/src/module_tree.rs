use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use bayou_interner::{Interner, Istr};
use bayou_ir::symbols::FuncId;
use bayou_session::sourcemap::SourceId;
use bayou_utils::{declare_key_type, keyvec::KeyVec};

use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath {
    components: Vec<Istr>,
}

impl ModulePath {
    pub fn root() -> Self {
        Self { components: vec![] }
    }

    pub fn push(&mut self, name: Istr) {
        self.components.push(name);
    }

    pub fn join(&self, name: Istr) -> Self {
        let mut components = self.components.clone();
        components.push(name);

        Self { components }
    }

    pub fn name(&self) -> Option<Istr> {
        self.components.last().copied()
    }

    pub fn components(&self) -> &[Istr] {
        &self.components
    }

    /// # Panics
    /// Calling [`DisplayModulePath::fmt`] panics or produces an invalid result if any of
    /// the path components are not from this interner.
    pub fn display<'a>(&'a self, interner: &'a Interner) -> DisplayModulePath<'a> {
        DisplayModulePath {
            path: self,
            interner,
        }
    }
}

pub struct DisplayModulePath<'a> {
    path: &'a ModulePath,
    interner: &'a Interner,
}

impl Display for DisplayModulePath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "package")?;

        for &component in &self.path.components {
            write!(f, "::{}", &self.interner[component])?;
        }

        Ok(())
    }
}

declare_key_type! { pub struct ModuleId; }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalId {
    Func(FuncId),
    Module(ModuleId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DuplicateGlobalError {
    pub first: GlobalId,
    pub second: GlobalId,
}

pub struct ParsedModule {
    pub source_id: SourceId,
    pub ast: ast::Module,
}

pub struct ModuleEntry {
    pub globals: HashMap<Istr, GlobalId>,
    pub path: ModulePath,

    pub parsed: Option<ParsedModule>,
}

pub struct ModuleTree {
    modules: KeyVec<ModuleId, ModuleEntry>,
    root_id: ModuleId,
}

impl Default for ModuleTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleTree {
    pub fn new() -> Self {
        let mut modules = KeyVec::new();

        let root_id = modules.insert(ModuleEntry {
            globals: HashMap::new(),
            path: ModulePath::root(),

            parsed: None,
        });

        Self { modules, root_id }
    }

    pub fn root_id(&self) -> ModuleId {
        self.root_id
    }

    /// # Panics
    /// Panics if the module is not in the tree.
    pub fn entry(&self, module: ModuleId) -> &ModuleEntry {
        &self.modules[module]
    }

    /// # Panics
    /// Panics if the module is not in the tree or this has already been called
    /// for this module.
    pub fn set_parsed_module(&mut self, module: ModuleId, parsed: ParsedModule) {
        let entry_parsed = &mut self.modules[module].parsed;

        assert!(
            entry_parsed.is_none(),
            "`ParsedModule` already set for this module."
        );

        *entry_parsed = Some(parsed);
    }

    /// # Panics
    /// Panics if the module is not in the tree.
    pub fn path(&self, module: ModuleId) -> &ModulePath {
        &self.modules[module].path
    }

    /// # Panics
    /// Panics if the module is not in the tree.
    pub fn get_global(&self, module: ModuleId, name: Istr) -> Option<GlobalId> {
        let module = &self.modules[module];
        module.globals.get(&name).copied()
    }

    /// # Panics
    /// Panics if the parent module is not in the tree.
    pub fn insert_module(
        &mut self,
        parent: ModuleId,
        name: Istr,
    ) -> Result<ModuleId, DuplicateGlobalError> {
        let path = self.modules[parent].path.join(name);

        let id = self.modules.insert(ModuleEntry {
            globals: HashMap::new(),
            path,

            parsed: None,
        });

        self.insert_global(parent, name, GlobalId::Module(id))?;

        Ok(id)
    }

    /// # Panics
    /// Panics if the module is not in the tree.
    pub fn insert_global(
        &mut self,
        module: ModuleId,
        name: Istr,
        symbol_id: GlobalId,
    ) -> Result<(), DuplicateGlobalError> {
        match self.modules[module].globals.insert(name, symbol_id) {
            None => Ok(()),
            Some(first) => Err(DuplicateGlobalError {
                first,
                second: symbol_id,
            }),
        }
    }
}
