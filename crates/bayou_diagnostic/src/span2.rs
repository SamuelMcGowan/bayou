use std::ops::Range;

pub trait Span {
    fn start(&self) -> usize;
    fn end(&self) -> usize;

    fn contains(&self, n: usize) -> bool {
        n >= self.start() && n < self.end()
    }

    fn len(&self) -> usize {
        self.end().saturating_sub(self.start())
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn as_range(&self) -> Range<usize> {
        self.start()..self.end()
    }
}

impl Span for Range<usize> {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl Span for (usize, usize) {
    fn start(&self) -> usize {
        self.0
    }

    fn end(&self) -> usize {
        self.1
    }
}

/// A simple span for use within this crate.
pub(crate) struct InternalSpan(usize, usize);

impl InternalSpan {
    pub fn new(s: impl Span) -> Self {
        Self(s.start(), s.end())
    }
}

impl Span for InternalSpan {
    fn start(&self) -> usize {
        self.0
    }

    fn end(&self) -> usize {
        self.1
    }
}
