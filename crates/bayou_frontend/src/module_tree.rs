use std::{collections::HashMap, ops::Deref};

use bayou_interner::Istr;
use bayou_ir::{
    symbols::{FuncId, Symbols},
    IdentWithSource,
};
use bayou_session::module_loader::ModulePath;
use bayou_utils::{declare_key_type, keyvec::KeyVec};

declare_key_type! {
    #[derive(serde::Serialize)]
    pub struct ModuleId;
}

#[derive(Debug, Clone, serde::Serialize)]
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
            ident: None,

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
        ident: IdentWithSource,
    ) -> Result<ModuleId, GlobalId> {
        let path = self.entries[parent].path.join(ident.istr);

        let id = self.entries.insert(ModuleEntry {
            ident: Some(ident),

            path,
            globals: HashMap::new(),
        });

        self.entry_mut(parent)
            .insert_global(ident.istr, GlobalId::Module(id))?;

        Ok(id)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleEntry {
    pub ident: Option<IdentWithSource>,

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
    pub fn insert_global(&mut self, name: Istr, global: GlobalId) -> Result<(), GlobalId> {
        match self.inner.globals.insert(name, global) {
            None => Ok(()),
            Some(first) => Err(first),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum GlobalId {
    Module(ModuleId),
    Func(FuncId),
}

impl GlobalId {
    pub fn as_func(self) -> Option<FuncId> {
        match self {
            Self::Module(_) => None,
            Self::Func(id) => Some(id),
        }
    }
}

pub fn get_global_ident(
    global: GlobalId,
    modules: &ModuleTree,
    symbols: &Symbols,
) -> Option<IdentWithSource> {
    match global {
        GlobalId::Module(id) => modules.entry(id).ident,
        GlobalId::Func(id) => Some(symbols.funcs[id].ident),
    }
}
