use std::num::NonZeroU32;
use std::ops::Range;

use ahash::RandomState;
use hashbrown::hash_table::Entry;
use hashbrown::HashTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interned {
    index: NonZeroU32,
    len: u32,
}

impl Interned {
    #[inline]
    fn as_range(&self) -> Range<usize> {
        let start = self.index.get() as usize - 1;
        start..(self.len as usize)
    }
}

pub struct Interner {
    random_state: RandomState,
    lookup: HashTable<Metadata>,
    data: String,
}

impl Interner {
    pub fn intern<Q>(&mut self, key: &str) -> Option<Interned> {
        let hash = self.random_state.hash_one(key);

        let entry = self.lookup.entry(
            hash,
            |&index| self.data.get(index.interned.as_range()) == Some(key),
            |&index| index.hash,
        );

        let index = match entry {
            Entry::Occupied(entry) => entry.get().interned,
            Entry::Vacant(entry) => {
                let interned = Interned {
                    index: NonZeroU32::new((self.data.len() + 1).try_into().ok()?)?,
                    len: key.len().try_into().ok()?,
                };

                self.data.push_str(key);
                entry.insert(Metadata { interned, hash });

                interned
            }
        };

        Some(index)
    }

    #[inline]
    pub fn get(&self, interned: Interned) -> Option<&str> {
        self.data.get(interned.as_range())
    }
}

#[derive(Clone, Copy)]
struct Metadata {
    interned: Interned,
    hash: u64,
}
