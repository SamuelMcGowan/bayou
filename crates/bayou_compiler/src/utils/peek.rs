pub trait Peek: Iterator {
    fn peek(&self) -> Option<Self::Item>;

    fn eat<P>(&mut self, pat: P) -> bool
    where
        Self::Item: PartialEq<P>,
    {
        match self.peek() {
            Some(item) if item == pat => {
                self.next();
                true
            }
            _ => false,
        }
    }

    fn at_end(&self) -> bool {
        self.peek().is_none()
    }
}

impl<P: Peek> Peek for &mut P {
    fn peek(&self) -> Option<Self::Item> {
        (**self).peek()
    }
}

impl<T> Peek for std::slice::Iter<'_, T> {
    fn peek(&self) -> Option<Self::Item> {
        self.clone().next()
    }
}

impl Peek for std::str::Chars<'_> {
    fn peek(&self) -> Option<Self::Item> {
        self.clone().next()
    }
}
