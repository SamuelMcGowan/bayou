pub trait Sources {
    type SourceId: Copy + Eq;
    type Source: Source;

    fn get_source(&self, id: Self::SourceId) -> Option<&Self::Source>;
}

pub trait Source {
    fn name(&self) -> &str;
    fn source(&self) -> &str;
}

impl<S: Source> Sources for Vec<S> {
    type SourceId = usize;
    type Source = S;

    fn get_source(&self, id: Self::SourceId) -> Option<&Self::Source> {
        self.get(id)
    }
}

impl Source for (&str, &str) {
    fn name(&self) -> &str {
        self.0
    }

    fn source(&self) -> &str {
        self.1
    }
}
