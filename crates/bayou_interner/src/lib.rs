//! A string interner.
//!
//! A string interner is a data structure commonly used in compilers and other contexts that need to
//! cheaply store and compare many often identical strings. "Interning" a string returns a pointer (or in
//! this implementation, an ID) that is cheap to copy and to perform string equality checks on. This is
//! achieved by deduplicating strings using an internal hash table.
//!
//! This string interner also stores all strings in a single bump-allocated buffer, courtesy of [`bumpalo`],
//! avoiding excessive allocation.
//!
//! I decided to represent interned strings with a 32-bit ID instead of a reference to avoid introducing lifetimes.
//! This does mean that accessing the underlying string requires calling a method on the interner, but this is a
//! single array lookup.
//!
//! # Example
//! ```rust
//! use bayou_interner::Interner;
//!
//! # fn main() {
//! let interner = Interner::new();
//!
//! let hello = interner.intern("hello");
//! let hello2 = interner.intern("hello");
//! let world = interner.intern("world");
//!
//! // Interned strings can be compared cheaply.
//! assert_ne!(hello, hello2);
//! assert_ne!(hello, world);
//!
//! // Getting the associated string for an interned string.
//! assert_eq!(interner.get_str(hello), Some("hello"));
//! # }
//! ```

mod arena;

use std::cell::RefCell;
use std::num::NonZeroU32;
use std::ops::Index;

use ahash::RandomState;
use arena::InternerArena;
use hashbrown::hash_table::Entry;
use hashbrown::HashTable;

/// An ID for an interned string. Cheap to copy, and to perform string equality checks on, as
/// internally it is simply a [`NonZeroU32`] ID. It can also be stored inside an [`Option`] for free
/// due to niche optimisation.
///
/// In order to get the associated string, the interned string must be looked up
/// in the interner it was created with.
///
/// Note that performing an equality check on interned strings from different
/// interners will give a nonsensical result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Istr(NonZeroU32);

impl Istr {
    #[inline]
    fn from_index(index: usize) -> Option<Self> {
        Some(Self(NonZeroU32::new(
            u32::try_from(index).ok()?.wrapping_add(1),
        )?))
    }

    #[inline]
    fn to_index(self) -> usize {
        self.0.get() as usize - 1
    }
}

#[derive(Clone, Copy)]
struct Metadata {
    interned: Istr,
    hash: u64,
}

#[derive(Default)]
struct Lookup {
    random_state: RandomState,
    table: HashTable<Metadata>,
}

/// Storage for interned strings.
#[derive(Default)]
pub struct Interner {
    lookup: RefCell<Lookup>,
    arena: InternerArena,
}

impl Interner {
    /// Create a new interner.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a string, returning an interned string that it is cheap to copy and
    /// perform equality checks on. Strings are only stored in the interner once, no
    /// matter how many times they are interned.
    ///
    ///
    /// ```rust
    /// # use bayou_interner::Interner;
    /// #
    /// # fn main() {
    /// let interner = Interner::new();
    ///
    /// let hello = interner.intern("hello");
    /// let hello2 = interner.intern("hello");
    /// let world = interner.intern("world");
    ///
    /// assert_eq!(hello, hello2);
    /// assert_ne!(hello, world);
    /// # }
    /// ```
    ///
    /// # Panics
    /// Panics if there are no more available IDs. An interner can store up
    /// to `u32::MAX - 1` strings before panicking.
    #[inline]
    pub fn intern(&self, key: &str) -> Istr {
        self.try_intern(key).expect("Too many interned strings")
    }

    /// Like [`Interner::intern`], but non-panicking in the case that there are no
    /// more available IDs.
    pub fn try_intern(&self, key: &str) -> Option<Istr> {
        let mut lookup = self.lookup.borrow_mut();

        let hash = lookup.random_state.hash_one(key);

        let entry = lookup.table.entry(
            hash,
            |metadata| self.arena.get(metadata.interned.to_index()) == Some(key),
            |metadata| metadata.hash,
        );

        let interned = match entry {
            Entry::Occupied(entry) => entry.get().interned,
            Entry::Vacant(entry) => {
                let index = self.arena.push_str(key);
                let interned = Istr::from_index(index)?;

                entry.insert(Metadata { interned, hash });

                interned
            }
        };

        Some(interned)
    }

    /// Get an interned string if this string is interned, otherwise return `None`.
    ///
    /// ```rust
    /// # use bayou_interner::{Interner, Istr};
    /// #
    /// # fn main() {
    /// let interner = Interner::new();
    ///
    /// let hello: Istr = interner.intern("hello");
    ///
    /// assert_eq!(interner.get_interned("hello"), Some(hello));
    /// assert_eq!(interner.get_interned("world"), None);
    /// # }
    /// ```
    pub fn get_interned(&self, key: &str) -> Option<Istr> {
        let lookup = self.lookup.borrow();

        let hash = lookup.random_state.hash_one(key);

        lookup
            .table
            .find(hash, |metadata| {
                self.arena.get(metadata.interned.to_index()) == Some(key)
            })
            .map(|metadata| metadata.interned)
    }

    /// Look up an interned string to get the associated string.
    ///
    /// Note that if the interned string was created by another interner
    /// this method will return an arbitrary string or `None`.
    ///
    /// If you know that an interned string was created by this interner, you can index into
    /// the interner instead. This is the same as calling this method, but panics if the
    /// interned string is not found in this interner instead of returning `None`.
    ///
    /// ```rust
    /// # use bayou_interner::Interner;
    /// #
    /// # fn main() {
    /// let interner = Interner::new();
    /// let interned = interner.intern("hello");
    ///
    /// assert_eq!(interner.get_str(interned), Some("hello"));
    /// # }
    /// ```
    #[inline]
    pub fn get_str(&self, interned: Istr) -> Option<&str> {
        self.arena.get(interned.to_index())
    }
}

impl Index<Istr> for Interner {
    type Output = str;

    #[inline]
    fn index(&self, interned: Istr) -> &Self::Output {
        self.get_str(interned).expect("String not in interner")
    }
}

#[test]
fn test_interner() {
    let interner = Interner::new();

    for n in 0..100 {
        let s = n.to_string();

        let a = interner.intern(&s);
        let b = interner.intern(&s);

        assert_eq!(a, b);
        assert_eq!(interner.get_str(a), Some(s.as_str()));
    }
}
