use std::{collections::HashMap, ops::Deref};

use bayou_interner::Istr;
use bayou_ir::symbols::FuncId;
use bayou_session::module_loader::ModulePath;
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ModuleEntry {
    pub path: ModulePath,
    pub globals: HashMap<Istr, GlobalId>,
}

#[derive(Debug)]
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
