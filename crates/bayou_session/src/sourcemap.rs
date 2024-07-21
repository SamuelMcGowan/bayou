use bayou_diagnostic::{
    sources::{Cached, SourceMap as _},
    span::Span,
};
use bayou_utils::keyvec::{declare_key_type, KeyVec};

declare_key_type! {
    #[derive(serde::Serialize)]
    pub struct SourceId;
}

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

    #[allow(clippy::missing_panics_doc)]
    pub fn insert_and_get(&mut self, source: Source) -> (SourceId, &Cached<Source>) {
        let id = self.insert(source);
        (id, self.get_source(id).unwrap())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct SourceSpan {
    pub span: Span,
    pub source_id: SourceId,
}

impl SourceSpan {
    pub fn new(span: Span, source_id: SourceId) -> Self {
        Self { span, source_id }
    }
}
