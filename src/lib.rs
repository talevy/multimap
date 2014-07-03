//! This crate provides the `MultiMap` type, a convenient wrapper around
//! HashMaps with multiple values per key

#![crate_id="multimap#0.0.1"]
#![crate_type="rlib"]
#![crate_type="dylib"]
#![warn(unnecessary_qualification, non_uppercase_statics,
        variant_size_difference, managed_heap_memory, unnecessary_typecast,
        missing_doc, unused_result)]

use std::collections::{Collection, HashMap, Mutable};
use std::default::Default;
use std::fmt::Show;
use std::fmt;
use std::hash::Hash;
use std::iter::Repeat;

/// A map containing multiple values per key by providing
/// a convenient wrapper around HashMap<K, Vec<V>>.
///
/// This multimap allows duplicate key-value pairs.
///
/// ```rust
/// # use multimap::MultiMap;
/// let mut data = MultiMap::new();
/// data.insert(1, 4);
/// data.insert(1, 8);
/// ```
pub struct MultiMap<K, V> {
     data: HashMap<K, Vec<V>>,
}

impl<K: Hash + Eq, V> Collection for MultiMap<K, V> {
    fn len(&self) -> uint {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<K: Hash + Eq, V> Mutable for MultiMap<K, V> {
    fn clear(&mut self) {
        self.data.clear();
    }
}

impl<K: Hash + Eq, V: PartialEq> PartialEq for MultiMap<K, V> {
    fn eq(&self, other: &MultiMap<K, V>) -> bool {
        self.data.eq(&other.data)
    }
}

impl<K: Hash + Eq + Show, V: Show> Show for MultiMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl<K: Clone + Eq + Hash, V: Clone + PartialEq> FromIterator<(K, V)> for MultiMap<K, V> {
    fn from_iter<I: Iterator<(K, V)>>(mut iter: I) -> MultiMap<K, V> {
        let (lower, _) = iter.size_hint();
        let mut map: MultiMap<K, V> = MultiMap::with_capacity_and_default_hasher(lower);
        for (k, v) in iter {
            map.insert(k, v);
        }

        map
    }
}

impl<K: Clone + Eq + Hash, V: Clone + PartialEq> MultiMap<K, V> {
    /// Constructs a new `MultiMap` with a default hasher and a specified 
    /// initial size.
    ///
    /// Currently, this is not public because it is to only be used by the 
    /// `FromIterator` implementation for `MultiMap`.
    fn with_capacity_and_default_hasher(capacity: uint) -> MultiMap<K, V> {
        MultiMap {
            data: HashMap::with_capacity_and_hasher(capacity, Default::default())
        }
    }

    /// Construct a new `MultiMap`.
    pub fn new() -> MultiMap<K, V> {
        MultiMap {
            data: HashMap::new()
        }
    }

    /// Retrieves a vector of values for the given key, failing if the 
    /// key is not present.
    pub fn get<'a>(&'a self, k: &K) -> &'a Vec<V> {
        self.data.get(k)
    }

    /// Retrieves a (mutable) vector of values for the given key, failing if the 
    /// key is not present.
    pub fn get_mut<'a>(&'a mut self, k: &K) -> &'a mut Vec<V> {
        self.data.get_mut(k)
    }

    /// Return true if the map contains a value for a specified key.
    pub fn contains_key(&self, k: &K) -> bool {
        self.data.contains_key(k)
    }

    /// WARNING: hack
    /// An iterator visiting all key-value pairs in arbitrary order
    /// Iterator element type is (K, V>).
    /// TODO(talevy): figure out a clean way to lazily iterate
    pub fn as_vec<'a>(&'a self) -> Vec<(K, V)> {
        let mut entries: Vec<(K, V)> = Vec::new();

        for (_k, v) in self.data.iter() {
            let rep = Repeat::new(_k);
            for (kk, vv) in rep.zip(v.iter()) {
                entries.push((kk.clone(), vv.clone()));
            }
        }
        entries
    }

    /// Inserts the specified key-value pair into the multimap.
    ///
    /// Duplicate key-value pairs are allowed. If (k,v) is already found 
    /// in the map, another will be added.
    #[inline]
    pub fn insert(&mut self, k: K, v: V) {
        if self.data.contains_key(&k) {
            self.data.get_mut(&k).push(v);
        } else {
            self.data.insert(k, vec!(v));
        }
    }

    /// Removes key and it's associated value from map.
    #[inline]
    pub fn remove(&mut self, k: &K) -> bool {
        self.data.remove(k)
    }

    /// Removes specified key-value pair from map.
    /// If no more values associated with specified key,
    /// that key will be removed from map.
    #[inline]
    pub fn remove_value(&mut self, k: &K, v: &V) -> bool {
        if self.data.contains_key(k) {
            let mut new_vec: Vec<V> = Vec::new();
            for val in self.get(k).iter() {
                if val != v {
                    new_vec.push(val.clone());
                }
            }
            if new_vec.is_empty() {
                self.remove(k);
            } else {
                *self.data.get_mut(k) = new_vec;
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::MultiMap;

    type TestMultiMap = MultiMap<uint, uint>;

    fn create_test_map() -> TestMultiMap {
        let mut map: MultiMap<uint, uint> = MultiMap::new();
        map.insert(1, 3);
        map.insert(1, 5);
        map.insert(1, 7);
        return map;
    }

    #[test]
    fn test_order() {
        let map = create_test_map();
        let vec = map.as_vec();
        let mut iter = vec.iter();
        assert_eq!(iter.next().unwrap(), &(1u, 3u));
        assert_eq!(iter.next().unwrap(), &(1u, 5u));
        assert_eq!(iter.next().unwrap(), &(1u, 7u));
    }

    #[test]
    fn test_remove() {
        let mut map = create_test_map();
        let key = &1;
        map.remove(key);
        assert!(!map.contains_key(key));
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_remove_value() {
        let mut map = create_test_map();

        map.remove_value(&1, &3);
        assert_eq!(map.get(&1).len(), 2);
        assert!(!map.get(&1).contains(&3));

        map.remove_value(&1, &5);
        assert_eq!(map.get(&1).len(), 1);
        assert!(!map.get(&1).contains(&5));

        map.remove_value(&1, &7);
        assert!(!map.contains_key(&1));
    }

    #[test]
    fn test_from_iterator() {
        // sorted input tuples
        let input = vec!((1u, 3u), (1u, 5u), (2u, 6u));
        let map: MultiMap<uint, uint> = input.iter().map(|x| *x).collect();
        let mut test = map.as_vec();

        test.sort();

        assert_eq!(input, test);
    }
}
