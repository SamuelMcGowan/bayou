use std::ops::Range;

#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, n: usize) -> bool {
        n >= self.start && n < self.end
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for Span {
    type Item = usize;
    type IntoIter = Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.start..self.end
    }
}

pub trait AsSpan {
    fn as_span(&self) -> Span;
}

impl AsSpan for Span {
    fn as_span(&self) -> Span {
        *self
    }
}

impl AsSpan for Range<usize> {
    fn as_span(&self) -> Span {
        Span::new(self.start, self.end)
    }
}

impl AsSpan for (usize, usize) {
    fn as_span(&self) -> Span {
        Span::new(self.0, self.1)
    }
}
