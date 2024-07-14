use std::{
    collections::HashMap,
    fmt::{self, Display},
    ops::Deref,
};

use bayou_interner::{Interner, Istr};
use bayou_ir::symbols::FuncId;
use bayou_utils::{declare_key_type, keyvec::KeyVec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalId {
    Module(ModuleId),
    Func(FuncId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DuplicateGlobalError {
    pub first: GlobalId,
    pub second: GlobalId,
}

declare_key_type! { pub struct ModuleId; }

pub struct ModuleTree {
    entries: KeyVec<ModuleId, ModuleEntry>,
    root_id: ModuleId,
}

impl Default for ModuleTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleTree {
    pub fn new() -> Self {
        let mut scopes = KeyVec::new();

        let root_id = scopes.insert(ModuleEntry {
            path: ModulePath::root(),
            globals: HashMap::new(),
        });

        Self {
            entries: scopes,
            root_id,
        }
    }

    pub fn root_id(&self) -> ModuleId {
        self.root_id
    }

    pub fn entry(&self, id: ModuleId) -> &ModuleEntry {
        &self.entries[id]
    }

    pub fn entry_mut(&mut self, id: ModuleId) -> ModuleEntryMut {
        ModuleEntryMut {
            inner: &mut self.entries[id],
        }
    }

    pub fn insert_module(
        &mut self,
        parent: ModuleId,
        name: Istr,
    ) -> Result<ModuleId, DuplicateGlobalError> {
        let path = self.entries[parent].path.join(name);

        let id = self.entries.insert(ModuleEntry {
            path,
            globals: HashMap::new(),
        });

        self.entry_mut(parent)
            .insert_global(name, GlobalId::Module(id))?;

        Ok(id)
    }
}

pub struct ModuleEntry {
    pub path: ModulePath,
    pub globals: HashMap<Istr, GlobalId>,
}

pub struct ModuleEntryMut<'a> {
    inner: &'a mut ModuleEntry,
}

impl Deref for ModuleEntryMut<'_> {
    type Target = ModuleEntry;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl ModuleEntryMut<'_> {
    pub fn insert_global(
        &mut self,
        name: Istr,
        global: GlobalId,
    ) -> Result<(), DuplicateGlobalError> {
        match self.inner.globals.insert(name, global) {
            None => Ok(()),
            Some(first) => Err(DuplicateGlobalError {
                first,
                second: global,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath {
    components: Vec<Istr>,
}

impl ModulePath {
    pub fn new(components: impl Into<Vec<Istr>>) -> Self {
        Self {
            components: components.into(),
        }
    }

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
