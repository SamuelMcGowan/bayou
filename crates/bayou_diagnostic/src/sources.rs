use std::path::{Path, PathBuf};

pub trait Sources {
    type SourceId: Copy + Eq + std::hash::Hash;
    type Source: Source;

    fn get_source(&self, id: Self::SourceId) -> Option<&Cached<Self::Source>>;
}

pub trait Source {
    fn name_str(&self) -> &str;
    fn path(&self) -> Option<&Path>;

    fn source_str(&self) -> &str;
}

impl<S: Source> Sources for Vec<Cached<S>> {
    type SourceId = usize;
    type Source = S;

    fn get_source(&self, id: Self::SourceId) -> Option<&Cached<Self::Source>> {
        self.get(id)
    }
}

impl Source for (String, String) {
    fn name_str(&self) -> &str {
        &self.0
    }

    fn path(&self) -> Option<&Path> {
        None
    }

    fn source_str(&self) -> &str {
        &self.1
    }
}

impl Source for (String, PathBuf, String) {
    fn name_str(&self) -> &str {
        &self.0
    }

    fn path(&self) -> Option<&Path> {
        Some(&self.1)
    }

    fn source_str(&self) -> &str {
        &self.2
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug)]
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
        } else {
            self.line_breaks.get(line - 1).map(|&byte| byte + 1)
        }
    }

    pub fn line_str(&self, index: usize) -> Option<&str> {
        let start = self.line_to_byte(index)?;
        let end = self
            .line_to_byte(index + 1)
            .unwrap_or(self.source_str().len());

        let s = &self.source_str()[start..end];
        let s = s.strip_suffix('\n').unwrap_or(s);
        let s = s.strip_suffix('\r').unwrap_or(s);

        Some(s)
    }

    pub fn num_lines(&self) -> usize {
        1 + self.line_breaks.len()
    }
}

impl<S: Source> Source for Cached<S> {
    fn name_str(&self) -> &str {
        self.source.name_str()
    }

    fn path(&self) -> Option<&Path> {
        self.source.path()
    }

    fn source_str(&self) -> &str {
        self.source.source_str()
    }
}

#[cfg(test)]
mod tests {
    use super::Cached;

    fn cached_str(s: impl Into<String>) -> Cached<(String, String)> {
        Cached::new(("sample".to_owned(), s.into()))
    }

    #[test]
    fn test_line_breaks() {
        let cached = cached_str("");
        assert_eq!(cached.byte_to_line_index(0), Some(0));
        assert_eq!(cached.byte_to_line_index(1), None);

        let cached = cached_str("\n");
        assert_eq!(cached.byte_to_line_index(0), Some(0));
        assert_eq!(cached.byte_to_line_index(1), Some(1));
        assert_eq!(cached.byte_to_line_index(2), None);

        let cached = cached_str("x\n");
        assert_eq!(cached.byte_to_line_index(0), Some(0));
        assert_eq!(cached.byte_to_line_index(1), Some(0));
        assert_eq!(cached.byte_to_line_index(2), Some(1));
        assert_eq!(cached.byte_to_line_index(3), None);

        let cached = cached_str("\nx");
        assert_eq!(cached.byte_to_line_index(0), Some(0));
        assert_eq!(cached.byte_to_line_index(1), Some(1));
        assert_eq!(cached.byte_to_line_index(2), Some(1));
        assert_eq!(cached.byte_to_line_index(3), None);
    }

    #[test]
    fn test_line_col() {
        let cached = cached_str("");
        assert_eq!(cached.byte_to_line_col(0), Some((1, 1)));
        assert_eq!(cached.byte_to_line_col(1), None);

        let cached = cached_str("\n");
        assert_eq!(cached.byte_to_line_col(0), Some((1, 1)));
        assert_eq!(cached.byte_to_line_col(1), Some((2, 1)));
        assert_eq!(cached.byte_to_line_col(2), None);

        let cached = cached_str("x\n");
        assert_eq!(cached.byte_to_line_col(0), Some((1, 1)));
        assert_eq!(cached.byte_to_line_col(1), Some((1, 2)));
        assert_eq!(cached.byte_to_line_col(2), Some((2, 1)));

        let cached = cached_str("\nx");
        assert_eq!(cached.byte_to_line_col(0), Some((1, 1)));
        assert_eq!(cached.byte_to_line_col(1), Some((2, 1)));
        assert_eq!(cached.byte_to_line_col(2), Some((2, 2)));
    }

    #[test]
    fn test_line_to_byte() {
        let cached = cached_str("");
        assert_eq!(cached.line_to_byte(0), Some(0));
        assert_eq!(cached.line_to_byte(1), None);

        let cached = cached_str("\n");
        assert_eq!(cached.line_to_byte(0), Some(0));
        assert_eq!(cached.line_to_byte(1), Some(1));
        assert_eq!(cached.line_to_byte(2), None);

        let cached = cached_str("x\n");
        assert_eq!(cached.line_to_byte(0), Some(0));
        assert_eq!(cached.line_to_byte(1), Some(2));
        assert_eq!(cached.line_to_byte(2), None);

        let cached = cached_str("\nx");
        assert_eq!(cached.line_to_byte(0), Some(0));
        assert_eq!(cached.line_to_byte(1), Some(1));
        assert_eq!(cached.line_to_byte(2), None);
    }

    #[test]
    fn test_line_str() {
        let cached = cached_str("");
        assert_eq!(cached.line_str(0), Some(""));
        assert_eq!(cached.line_str(1), None);

        let cached = cached_str("\n");
        assert_eq!(cached.line_str(0), Some(""));
        assert_eq!(cached.line_str(1), Some(""));
        assert_eq!(cached.line_str(2), None);

        let cached = cached_str("x\n");
        assert_eq!(cached.line_str(0), Some("x"));
        assert_eq!(cached.line_str(1), Some(""));
        assert_eq!(cached.line_str(2), None);

        let cached = cached_str("\nx");
        assert_eq!(cached.line_str(0), Some(""));
        assert_eq!(cached.line_str(1), Some("x"));
        assert_eq!(cached.line_str(2), None);
    }
}
