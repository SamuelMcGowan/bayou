pub trait Sources {
    type SourceId: Copy + Eq;
    type Source: Source;

    fn get_source(&self, id: Self::SourceId) -> Option<&Cached<Self::Source>>;
}

pub trait Source {
    fn name_str(&self) -> &str;
    fn source_str(&self) -> &str;
}

impl<S: Source> Sources for Vec<Cached<S>> {
    type SourceId = usize;
    type Source = S;

    fn get_source(&self, id: Self::SourceId) -> Option<&Cached<Self::Source>> {
        self.get(id)
    }
}

impl Source for (&str, &str) {
    fn name_str(&self) -> &str {
        self.0
    }

    fn source_str(&self) -> &str {
        self.1
    }
}

pub struct Cached<S: Source> {
    source: S,
    line_breaks: Vec<usize>,
}

impl<S: Source> Cached<S> {
    pub fn new(source: S) -> Self {
        let source_str = source.source_str();
        let line_breaks = source_str
            .char_indices()
            .filter_map(|(i, ch)| (ch == '\n').then_some(i))
            .collect();

        Self {
            source,
            line_breaks,
        }
    }

    pub fn as_source(&self) -> &S {
        &self.source
    }

    pub fn byte_to_line_col(&self, byte: usize) -> Option<(usize, usize)> {
        let line = self.byte_to_line_index(byte)?;

        let line_start = self.line_to_byte(line)?;
        let col = byte - line_start;

        Some((line + 1, col + 1))
    }

    pub fn byte_to_line_index(&self, byte: usize) -> Option<usize> {
        if byte > self.source_str().len() {
            return None;
        }

        match self.line_breaks.binary_search(&byte) {
            Ok(line) | Err(line) => Some(line),
        }
    }

    pub fn line_to_byte(&self, line: usize) -> Option<usize> {
        if line == 0 {
            Some(0)
        } else if line == self.line_breaks.len() {
            Some(self.source_str().len())
        } else {
            self.line_breaks.get(line - 1).copied()
        }
    }
}

impl<S: Source> Source for Cached<S> {
    fn name_str(&self) -> &str {
        self.source.name_str()
    }

    fn source_str(&self) -> &str {
        self.source.source_str()
    }
}

#[test]
fn test_line_breaks() {
    let cached = Cached::new(("a", ""));
    assert_eq!(cached.byte_to_line_index(0), Some(0));
    assert_eq!(cached.byte_to_line_index(1), None);

    let cached = Cached::new(("a", "\n"));
    assert_eq!(cached.byte_to_line_index(0), Some(0));
    assert_eq!(cached.byte_to_line_index(1), Some(1));
    assert_eq!(cached.byte_to_line_index(2), None);

    let cached = Cached::new(("a", "x\n"));
    assert_eq!(cached.byte_to_line_index(0), Some(0));
    assert_eq!(cached.byte_to_line_index(1), Some(0));
    assert_eq!(cached.byte_to_line_index(2), Some(1));
    assert_eq!(cached.byte_to_line_index(3), None);

    let cached = Cached::new(("a", "\nx"));
    assert_eq!(cached.byte_to_line_index(0), Some(0));
    assert_eq!(cached.byte_to_line_index(1), Some(1));
    assert_eq!(cached.byte_to_line_index(2), Some(1));
    assert_eq!(cached.byte_to_line_index(3), None);
}
