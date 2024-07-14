use std::{
    collections::HashMap,
    fmt::{self, Display},
    ops::Deref,
};

use bayou_interner::{Interner, Istr};
use bayou_ir::symbols::FuncId;
use bayou_utils::{declare_key_type, keyvec::KeyVec};

declare_key_type! { pub struct GlobalScopeId; }

pub struct GlobalScopeTree {
    scopes: KeyVec<GlobalScopeId, GlobalScope>,
    root_id: GlobalScopeId,
}

impl Default for GlobalScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalScopeTree {
    pub fn new() -> Self {
        let mut scopes = KeyVec::new();

        let root_id = scopes.insert(GlobalScope {
            path: GlobalPath::root(),
            globals: HashMap::new(),
        });

        Self { scopes, root_id }
    }

    pub fn root_id(&self) -> GlobalScopeId {
        self.root_id
    }

    pub fn scope(&self, id: GlobalScopeId) -> &GlobalScope {
        &self.scopes[id]
    }

    pub fn scope_mut(&mut self, id: GlobalScopeId) -> GlobalScopeMut {
        GlobalScopeMut {
            inner: &mut self.scopes[id],
        }
    }

    pub fn insert_scope(
        &mut self,
        parent: GlobalScopeId,
        name: Istr,
    ) -> Result<GlobalScopeId, DuplicateGlobalError> {
        let path = self.scopes[parent].path.join(name);

        let id = self.scopes.insert(GlobalScope {
            path,
            globals: HashMap::new(),
        });

        self.scope_mut(parent)
            .insert_global(name, GlobalId::Scope(id))?;

        Ok(id)
    }
}

pub struct GlobalScope {
    path: GlobalPath,
    globals: HashMap<Istr, GlobalId>,
}

pub struct GlobalScopeMut<'a> {
    inner: &'a mut GlobalScope,
}

impl Deref for GlobalScopeMut<'_> {
    type Target = GlobalScope;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl GlobalScopeMut<'_> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalId {
    Scope(GlobalScopeId),
    Func(FuncId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DuplicateGlobalError {
    pub first: GlobalId,
    pub second: GlobalId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalPath {
    components: Vec<Istr>,
}

impl GlobalPath {
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
    pub fn display<'a>(&'a self, interner: &'a Interner) -> DisplayGlobalPath<'a> {
        DisplayGlobalPath {
            path: self,
            interner,
        }
    }
}

pub struct DisplayGlobalPath<'a> {
    path: &'a GlobalPath,
    interner: &'a Interner,
}

impl Display for DisplayGlobalPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "package")?;

        for &component in &self.path.components {
            write!(f, "::{}", &self.interner[component])?;
        }

        Ok(())
    }
}
