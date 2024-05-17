use std::borrow::Borrow;
use std::hash::Hash;
use std::num::NonZeroUsize;

use ahash::RandomState;
use hashbrown::hash_table::Entry;
use hashbrown::HashTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interned(NonZeroUsize);

pub struct Interner<T> {
    random_state: RandomState,
    lookup: HashTable<Index>,

    interned_entries: Vec<T>,
}

impl<T> Interner<T> {
    pub fn intern<Q>(&mut self, key: &Q) -> Interned
    where
        Q: ToOwned<Owned = T> + Hash + Eq + ?Sized,
        T: Borrow<Q>,
    {
        let hash = self.random_state.hash_one(key);

        let entry = self.lookup.entry(
            hash,
            |&index| self.interned_entries[index.index].borrow() == key,
            |&index| index.hash,
        );

        let index = match entry {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let index = Index {
                    index: self.interned_entries.len(),
                    hash,
                };

                self.interned_entries.push(key.to_owned());
                entry.insert(index);

                index
            }
        };

        Interned(NonZeroUsize::new(index.index.wrapping_add(1)).unwrap())
    }

    #[inline]
    pub fn get(&self, interned: Interned) -> Option<&T> {
        self.interned_entries.get(interned.0.get() - 1)
    }
}

#[derive(Clone, Copy)]
struct Index {
    index: usize,
    hash: u64,
}
