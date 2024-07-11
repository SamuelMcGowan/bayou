use std::{
    collections::HashMap,
    fmt::{self, Display},
    ops::Deref,
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

impl ModuleEntry {
    pub fn get_global(&self, name: Istr) -> Option<GlobalId> {
        self.globals.get(&name).copied()
    }
}

pub struct ModuleEntryMut<'a> {
    entry: &'a mut ModuleEntry,
}

impl Deref for ModuleEntryMut<'_> {
    type Target = ModuleEntry;

    fn deref(&self) -> &Self::Target {
        self.entry
    }
}

impl ModuleEntryMut<'_> {
    /// # Panics
    /// Panics if this method has already been called for this module.
    pub fn set_parsed(&mut self, parsed: ParsedModule) {
        assert!(
            self.entry.parsed.is_none(),
            "`ParsedModule` already provided for this module"
        );

        self.entry.parsed = Some(parsed);
    }

    pub fn insert_global(
        &mut self,
        name: Istr,
        global: GlobalId,
    ) -> Result<(), DuplicateGlobalError> {
        match self.entry.globals.insert(name, global) {
            None => Ok(()),
            Some(first) => Err(DuplicateGlobalError {
                first,
                second: global,
            }),
        }
    }
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
    /// Panics if the module is not in the tree.
    pub fn entry_mut(&mut self, module: ModuleId) -> ModuleEntryMut {
        ModuleEntryMut {
            entry: &mut self.modules[module],
        }
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

        self.entry_mut(parent)
            .insert_global(name, GlobalId::Module(id))?;

        Ok(id)
    }
}
