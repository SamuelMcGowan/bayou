use std::collections::HashMap;

use bayou_interner::Istr;
use bayou_ir::symbols::FuncId;
use bayou_utils::{declare_key_type, keyvec::KeyVec};

declare_key_type! { pub struct ModuleId; }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DuplicateGlobal {
    pub first: GlobalId,
    pub second: GlobalId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalId {
    Func(FuncId),
    Module(ModuleId),
}

pub struct ModuleGlobalLookup {
    modules: KeyVec<ModuleId, ModuleInfo>,
    root: ModuleId,
}

impl Default for ModuleGlobalLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleGlobalLookup {
    pub fn new() -> Self {
        let mut modules = KeyVec::new();
        let root = modules.insert(ModuleInfo::default());

        Self { modules, root }
    }

    pub fn root_module(&self) -> ModuleId {
        self.root
    }

    /// # Panics
    /// Panics if the parent module is not in the tree.
    pub fn module(&mut self, parent: ModuleId, name: Istr) -> Result<ModuleId, DuplicateGlobal> {
        let module = self.modules.insert(ModuleInfo::default());
        self.insert_global(parent, name, GlobalId::Module(module))?;
        Ok(module)
    }

    /// # Panics
    /// Panics if the module is not in the tree.
    pub fn insert_global(
        &mut self,
        module: ModuleId,
        name: Istr,
        symbol_id: GlobalId,
    ) -> Result<(), DuplicateGlobal> {
        match self.modules[module].global_lookup.insert(name, symbol_id) {
            None => Ok(()),
            Some(first) => Err(DuplicateGlobal {
                first,
                second: symbol_id,
            }),
        }
    }

    /// # Panics
    /// Panics if the module is not in the tree.
    pub fn lookup_global(&self, module: ModuleId, name: Istr) -> Option<GlobalId> {
        let module = &self.modules[module];
        module.global_lookup.get(&name).copied()
    }
}

#[derive(Default)]
struct ModuleInfo {
    global_lookup: HashMap<Istr, GlobalId>,
}
