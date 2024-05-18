use bumpalo::Bump;

#[derive(Default)]
pub struct InternerArena {
    vec: Vec<*const str>,
    alloc: Bump,
}

impl InternerArena {
    #[inline]
    pub fn push_str(&mut self, s: &str) -> usize {
        let index = self.vec.len();

        let s = &*self.alloc.alloc_str(s);
        self.vec.push(s as *const str);

        index
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&str> {
        self.vec
            .get(index)
            // Safety:
            // - We don't deallocate or move the allocated strings as long as
            //   `self` is alive.
            // - There are no mutable references to the string.
            .map(|&ptr| unsafe { &*ptr })
    }
}
