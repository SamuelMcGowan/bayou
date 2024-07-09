use std::{collections::HashMap, fmt, fs, io, path::PathBuf};

use bayou_interner::{Interner, Istr};

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

impl fmt::Display for DisplayModulePath<'_> {
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

    match module_path.components.as_slice() {
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
