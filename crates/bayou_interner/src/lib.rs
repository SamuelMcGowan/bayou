mod arena;

use std::num::NonZeroU32;

use ahash::RandomState;
use arena::InternerArena;
use hashbrown::hash_table::Entry;
use hashbrown::HashTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub struct Interner {
    random_state: RandomState,
    lookup: HashTable<Metadata>,

    arena: InternerArena,
}

impl Interner {
    pub fn intern(&mut self, key: &str) -> Option<Interned> {
        let hash = self.random_state.hash_one(key);

        let entry = self.lookup.entry(
            hash,
            |&metadata| self.arena.get(metadata.interned.to_index()) == Some(key),
            |&metadata| metadata.hash,
        );

        let interned = match entry {
            Entry::Occupied(entry) => entry.get().interned,
            Entry::Vacant(entry) => {
                let index = self.arena.push_str(key);
                let interned = Interned::from_index(index).expect("Too many interned strings");

                entry.insert(Metadata { interned, hash });

                interned
            }
        };

        Some(interned)
    }

    #[inline]
    pub fn get(&self, interned: Interned) -> Option<&str> {
        self.arena.get(interned.to_index())
    }
}
