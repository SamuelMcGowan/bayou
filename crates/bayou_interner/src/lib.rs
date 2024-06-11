mod arena;

use std::cell::RefCell;
use std::num::NonZeroU32;

use ahash::RandomState;
use arena::InternerArena;
use hashbrown::hash_table::Entry;
use hashbrown::HashTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interned(NonZeroU32);

impl Interned {
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
    interned: Interned,
    hash: u64,
}

#[derive(Default)]
struct Lookup {
    random_state: RandomState,
    table: HashTable<Metadata>,
}

#[derive(Default)]
pub struct Interner {
    lookup: RefCell<Lookup>,
    arena: InternerArena,
}

impl Interner {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn intern(&self, key: &str) -> Interned {
        self.try_intern(key).expect("Too many interned strings")
    }

    pub fn try_intern(&self, key: &str) -> Option<Interned> {
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
                let interned = Interned::from_index(index)?;

                entry.insert(Metadata { interned, hash });

                interned
            }
        };

        Some(interned)
    }

    /// Get an interned string if it is present.
    pub fn get_str(&self, key: &str) -> Option<Interned> {
        let lookup = self.lookup.borrow();

        let hash = lookup.random_state.hash_one(key);

        lookup
            .table
            .find(hash, |metadata| {
                self.arena.get(metadata.interned.to_index()) == Some(key)
            })
            .map(|metadata| metadata.interned)
    }

    #[inline]
    pub fn get(&self, interned: Interned) -> &str {
        self.try_get(interned).expect("String not in interner")
    }

    #[inline]
    pub fn try_get(&self, interned: Interned) -> Option<&str> {
        self.arena.get(interned.to_index())
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
        assert_eq!(interner.get(a), s.as_str());
    }
}
