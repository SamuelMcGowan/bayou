use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, Index, IndexMut};

pub struct KeyVec<K, V> {
    inner: Vec<V>,
    _phantom: PhantomData<*const K>,
}

impl<K: Key, V> KeyVec<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn insert(&mut self, value: V) -> K {
        let key = K::from_usize(self.inner.len());
        self.inner.push(value);
        key
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.inner.get(key.as_usize())
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner.get_mut(key.as_usize())
    }
}

impl<K, V> Default for KeyVec<K, V> {
    fn default() -> Self {
        Self {
            inner: vec![],
            _phantom: PhantomData,
        }
    }
}

impl<K, V> Deref for KeyVec<K, V> {
    type Target = [V];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K: Key, V> Index<K> for KeyVec<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(key).expect("key not found")
    }
}

impl<K: Key, V> IndexMut<K> for KeyVec<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(key).expect("key not found")
    }
}

impl<K, V> IntoIterator for KeyVec<K, V> {
    type Item = V;

    type IntoIter = std::vec::IntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<K: Key + fmt::Debug, V: fmt::Debug> fmt::Debug for KeyVec<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(
                self.inner
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (K::from_usize(i), v)),
            )
            .finish()
    }
}

impl<K, V: Clone> Clone for KeyVec<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<K, V: PartialEq> PartialEq for KeyVec<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

pub trait Key: Copy {
    fn from_usize(n: usize) -> Self;
    fn as_usize(&self) -> usize;
}

macro_rules! declare_key_type {
(
    $v:vis struct $i:ident;
) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
    $v struct $i(pub usize);

    impl $crate::utils::keyvec::Key for $i {
        fn from_usize(n: usize) -> Self {
            Self(n)
        }

        fn as_usize(&self) -> usize {
            self.0
        }
    }
};
}
pub(crate) use declare_key_type;
