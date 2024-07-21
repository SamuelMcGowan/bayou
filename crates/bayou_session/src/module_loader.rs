use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs, io,
    path::PathBuf,
};

use bayou_diagnostic::Snippet;
use bayou_interner::{Interner, Istr};
use serde::ser::SerializeStruct;

use crate::{sourcemap::SourceSpan, Diagnostic, IntoDiagnostic};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
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

    #[must_use]
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

#[derive(Debug, Clone)]
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
    fn load_module(
        &self,
        path: &ModulePath,
        interner: &Interner,
    ) -> Result<String, ModuleLoaderError>;
}

#[derive(Debug)]
pub struct ModuleLoaderError {
    pub path: ModulePath,
    pub cause: Option<Box<dyn std::error::Error>>,
}

impl IntoDiagnostic<(Option<SourceSpan>, &Interner)> for ModuleLoaderError {
    fn into_diagnostic(
        self,
        &(source_span, interner): &(Option<SourceSpan>, &Interner),
    ) -> Diagnostic {
        let mut diagnostic = Diagnostic::error().with_message(format!(
            "couldn't load module `{}`",
            self.path.display(interner)
        ));

        if let Some(source_span) = source_span {
            diagnostic = diagnostic.with_snippet(Snippet::primary(
                "this module",
                source_span.source_id,
                source_span.span,
            ));
        }

        if let Some(cause) = &self.cause {
            diagnostic = diagnostic.with_note(cause.to_string());
        }

        diagnostic
    }
}

impl serde::Serialize for ModuleLoaderError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ModuleLoaderError", 2)?;

        s.serialize_field("path", &self.path)?;
        s.serialize_field("cause", &self.cause.as_ref().map(|cause| cause.to_string()))?;

        s.end()
    }
}

#[derive(Debug, Clone)]
pub struct FsLoader {
    pub root_dir: PathBuf,
}

impl ModuleLoader for FsLoader {
    fn load_module(
        &self,
        path: &ModulePath,
        interner: &Interner,
    ) -> Result<String, ModuleLoaderError> {
        let pathbuf = module_path_to_pathbuf(path, &self.root_dir, interner);
        fs::read_to_string(&pathbuf).map_err(|io_error| ModuleLoaderError {
            path: path.clone(),
            cause: Some(Box::new(FsLoaderError { pathbuf, io_error })),
        })
    }
}

#[derive(thiserror::Error, Debug)]
#[error("error reading file `{}`: {io_error}", pathbuf.display())]
pub struct FsLoaderError {
    pathbuf: PathBuf,
    io_error: io::Error,
}

#[derive(Debug, Clone)]
pub struct HashMapLoader {
    pub modules: HashMap<String, String>,
}

impl ModuleLoader for HashMapLoader {
    fn load_module(
        &self,
        path: &ModulePath,
        interner: &Interner,
    ) -> Result<String, ModuleLoaderError> {
        let path_str = path.display(interner).to_string();
        self.modules
            .get(&path_str)
            .cloned()
            .ok_or(ModuleLoaderError {
                path: path.clone(),
                cause: None,
            })
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
