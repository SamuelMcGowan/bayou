use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs, io,
    path::PathBuf,
};

use bayou_interner::{Interner, Istr};

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

pub trait ModuleLoader {
    type Error;

    fn load_module(&self, path: &ModulePath, interner: &Interner) -> Result<String, Self::Error>;
}

pub struct FsLoader {
    pub root_dir: PathBuf,
}

impl ModuleLoader for FsLoader {
    type Error = io::Error;

    fn load_module(&self, path: &ModulePath, interner: &Interner) -> Result<String, Self::Error> {
        let path = module_path_to_pathbuf(path, &self.root_dir, interner);
        fs::read_to_string(path)
    }
}

pub struct HashMapLoader {
    pub modules: HashMap<ModulePath, String>,
}

impl ModuleLoader for HashMapLoader {
    type Error = ();

    fn load_module(&self, path: &ModulePath, _interner: &Interner) -> Result<String, Self::Error> {
        self.modules.get(path).cloned().ok_or(())
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bayou_interner::Interner;

    use super::{module_path_to_pathbuf, ModulePath};

    #[test]
    fn module_paths() {
        let root_dir = PathBuf::from("source_dir");
        let interner = Interner::new();

        assert_eq!(
            module_path_to_pathbuf(&ModulePath::new([]), &root_dir, &interner),
            PathBuf::from("source_dir/main.by")
        );

        assert_eq!(
            module_path_to_pathbuf(
                &ModulePath::new([interner.intern("foo")]),
                &root_dir,
                &interner
            ),
            PathBuf::from("source_dir/foo.by")
        );

        assert_eq!(
            module_path_to_pathbuf(
                &ModulePath::new([interner.intern("foo"), interner.intern("bar")]),
                &root_dir,
                &interner
            ),
            PathBuf::from("source_dir/foo/bar.by")
        );
    }
}
