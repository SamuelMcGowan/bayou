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

    pub fn iter_mut(&mut self) -> std::slice::IterMut<V> {
        self.inner.iter_mut()
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

impl<'a, K, V> IntoIterator for &'a KeyVec<K, V> {
    type Item = &'a V;
    type IntoIter = std::slice::Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut KeyVec<K, V> {
    type Item = &'a mut V;
    type IntoIter = std::slice::IterMut<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
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

#[cfg(feature = "serialize")]
impl<K: Key + serde::Serialize, V: serde::Serialize> serde::Serialize for KeyVec<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut s = serializer.serialize_map(Some(self.len()))?;

        for (index, value) in self.iter().enumerate() {
            let key = K::from_usize(index);
            s.serialize_entry(&key, value)?;
        }

        s.end()
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

// TODO: use `NonZero...`` integers and allow using u32s
#[macro_export]
macro_rules! declare_key_type {
    ($(#[$meta:meta])* $v:vis struct $i:ident;) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $v struct $i(pub usize);

        impl $crate::keyvec::Key for $i {
            fn from_usize(n: usize) -> Self {
                Self(n)
            }

            fn as_usize(&self) -> usize {
                self.0
            }
        }
    };
}

pub use crate::declare_key_type;
