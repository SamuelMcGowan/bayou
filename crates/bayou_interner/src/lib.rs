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
    lookup: HashTable<usize>,

    interned_entries: Vec<InternedEntry<T>>,
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
            |&index| self.interned_entries[index].value.borrow() == key,
            |&index| self.interned_entries[index].hash,
        );

        let index = match entry {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let index = self.interned_entries.len();

                self.interned_entries.push(InternedEntry {
                    value: key.to_owned(),
                    hash,
                });
                entry.insert(index);

                index
            }
        };

        Interned(NonZeroUsize::new(index.wrapping_add(1)).unwrap())
    }

    #[inline]
    pub fn get(&self, interned: Interned) -> Option<&T> {
        self.interned_entries
            .get(interned.0.get() - 1)
            .map(|e| &e.value)
    }
}

struct InternedEntry<T> {
    value: T,
    hash: u64,
}
