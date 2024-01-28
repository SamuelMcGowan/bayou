use bayou_diagnostic::sources::Cached;

use crate::utils::keyvec::{declare_key_type, KeyVec};

declare_key_type! { pub struct SourceId; }

#[derive(Default, Debug, Clone)]
pub struct SourceMap {
    inner: KeyVec<SourceId, Cached<Source>>,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub name: String,
    pub source: String,
}

impl Source {
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
        }
    }
}

impl SourceMap {
    pub fn insert(&mut self, source: Source) -> SourceId {
        self.inner.insert(Cached::new(source))
    }
}

impl bayou_diagnostic::sources::SourceMap for SourceMap {
    type SourceId = SourceId;
    type Source = Source;

    fn get_source(&self, id: Self::SourceId) -> Option<&Cached<Self::Source>> {
        self.inner.get(id)
    }
}

impl bayou_diagnostic::sources::Source for Source {
    fn name_str(&self) -> &str {
        &self.name
    }

    fn path(&self) -> Option<&std::path::Path> {
        None
    }

    fn source_str(&self) -> &str {
        &self.source
    }
}
