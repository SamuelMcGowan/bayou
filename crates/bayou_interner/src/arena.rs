use std::cell::RefCell;

use bumpalo::Bump;

#[derive(Default)]
pub struct InternerArena {
    // Borrows can't fail since borrows are encapsulated.
    // This would change if we changed to being multithreaded.
    vec: RefCell<Vec<*const str>>,
    alloc: Bump,
}

// Safety:
// - The strings point to allocated strings that are never mutated,
//   so race conditions are impossible.
unsafe impl Send for InternerArena {}

impl InternerArena {
    #[inline]
    pub fn push_str(&self, s: &str) -> usize {
        let mut v = self.vec.borrow_mut();

        let index = v.len();

        let s = &*self.alloc.alloc_str(s);
        v.push(s as *const str);

        index
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&str> {
        let ptr = *self.vec.borrow().get(index)?;

        // Safety:
        // - We don't deallocate or move the allocated strings as long as
        //   `self` is alive.
        // - The strings are never mutated.
        Some(unsafe { &*ptr })
    }
}
