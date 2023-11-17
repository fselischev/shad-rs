#![forbid(unsafe_code)]

use std::{borrow::Borrow, iter::FromIterator, ops::Index};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FlatMap<K, V>(Vec<(K, V)>);

impl<K: Ord, V> FlatMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        self.0.as_slice()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let pos = match self.0.binary_search_by(|pair| pair.0.cmp(&key)) {
            Ok(pos) => return Some(std::mem::replace(&mut self.0[pos].1, value)),
            Err(pos) => pos,
        };

        self.0.insert(pos, (key, value));
        None
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>, // means K can be borrowed as &Q
        Q: Ord + ?Sized,
    {
        self.0
            .iter()
            .find(|(k, _)| k.borrow() == key)
            .map(|(_, v)| v)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        Some(
            self.0
                .remove(self.0.iter().position(|(k, _)| k.borrow() == key)?)
                .1,
        )
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        Some(
            self.0
                .remove(self.0.iter().position(|(k, _)| k.borrow() == key)?),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<K, V, Q> Index<&Q> for FlatMap<K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect("index should always be valid")
    }
}

impl<K: Ord, V> Extend<(K, V)> for FlatMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K: Ord, V> From<Vec<(K, V)>> for FlatMap<K, V> {
    fn from(value: Vec<(K, V)>) -> Self {
        Self::from_iter(value)
    }
}

impl<K: Ord, V> From<FlatMap<K, V>> for Vec<(K, V)> {
    fn from(value: FlatMap<K, V>) -> Self {
        Self::from_iter(value)
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for FlatMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut flat_map = FlatMap::new();
        flat_map.extend(iter);
        flat_map
    }
}

impl<K: Ord, V> IntoIterator for FlatMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
