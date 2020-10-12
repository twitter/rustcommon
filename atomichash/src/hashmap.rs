use crate::entry::{Entry, RawEntry};

use ahash::RandomState;

use std::borrow::Borrow;
use std::hash::{BuildHasher, Hasher};

pub struct AtomicHashMap<K, V> {
    hashers: [RandomState; 4],
    data: Vec<Entry<K, V>>,
}

impl<K, V> AtomicHashMap<K, V>
where
    K: std::cmp::PartialEq + std::hash::Hash + std::clone::Clone + std::fmt::Display,
{
    /// Allocate a new map with the ability to store up to the specified number
    /// of items. Depending on the keys, and overall size of the map, you may
    /// need to size the map to hold more keys than you have to ensure that
    /// inserts will be successful.
    pub fn with_capacity(items: usize) -> Self {
        let hashers = [
            RandomState::with_seeds(
                0xbb8c484891ec6c86,
                0x0522a25ae9c769f9,
                0x5b61bed2f4aed656,
                0xfdc618b31537d9ce,
            ),
            RandomState::with_seeds(
                0x8311d8f153515ff4,
                0xd22e51032364b4d3,
                0x7df7c397f82f015a,
                0x1a7a95f345f35b1f,
            ),
            RandomState::with_seeds(
                0x1bb782fb90137932,
                0x82bdf5530d94544e,
                0x040193377aba6b8b,
                0x7180722ed7f32bd8,
            ),
            RandomState::with_seeds(
                0xba4b6fc9f600b396,
                0x9579f32609013d9f,
                0x0f0742982048fcb2,
                0xf149d7b74b2b4dbf,
            ),
        ];
        let size = items.next_power_of_two();
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(Entry::empty())
        }
        Self { hashers, data }
    }

    /// Try to add a new key-value pair to the map. This function returns a
    /// error variant containing a key-value pair if the insert failed.
    /// *NOTE*: it is possible the key-value pair returned won't be the same
    /// pair as those inserted. This edge-case is possible if another thread
    /// raced our thread in-between checking an existing entry and performing
    /// the pointer swap.
    pub fn insert(&self, key: K, value: V) -> Result<(), (K, V)> {
        let mut positions = [0; 4];

        for (hash_id, position) in positions.iter_mut().enumerate() {
            let mut hasher = self.hashers[hash_id].build_hasher();
            key.hash(&mut hasher);
            *position = (hasher.finish() as usize) & (self.data.len() - 1);

            // check if it's a insert of an existing key
            if let Some(existing) = self.data[*position].load() {
                if existing.key == key {
                    let new = RawEntry::new(key.clone(), value);
                    if let Some(previous) = self.data[*position].swap(new) {
                        if previous.key != key {
                            // we raced and need to try to put the key-value pair
                            // back
                            return self.insert(previous.key.clone(), previous.value);
                        }
                    }
                    return Ok(());
                }
            }
        }

        // try to look for an empty entry in the table and insert the new entry
        // there
        for position in &positions {
            // if it's empty, this might be
            if self.data[*position].is_empty() {
                let new = RawEntry::new(key.clone(), value);
                if let Some(previous) = self.data[*position].swap(new) {
                    // we either replaced an entry for the same key, or we raced
                    // check the key to figure out which, and reinsert the previous
                    // entry if it was just a race
                    if previous.key != key {
                        // we raced and need to try to put the key-value pair
                        // back
                        return self.insert(previous.key.clone(), previous.value);
                    }
                }
                return Ok(());
            }
        }

        // there were no empty positions available for this key, we need to try
        // to find a key-value pair
        for position in &positions {
            // for each position the new key hashes to, we check if the current
            // key could map to an empty entry in the table. if it does, we will
            // swap the new key into the position
            if let Some(current_key) = self.data[*position].load().map(|v| v.key.clone()) {
                // eprintln!("considering moving {} out of position: {}", current_key, position);
                for hash_id in 0..4 {
                    let mut hasher = self.hashers[hash_id].build_hasher();
                    current_key.hash(&mut hasher);
                    let next_position = (hasher.finish() as usize) & (self.data.len() - 1);

                    // if we find a vacant entry, we take it
                    if self.data[next_position].is_empty() {
                        let new = RawEntry::new(key, value);
                        if let Some(current) = self.data[*position].swap(new) {
                            // but we may have raced, so check
                            if let Some(next) = self.data[next_position]
                                .swap(RawEntry::new(current.key.clone(), current.value))
                            {
                                // eprintln!("swapped into position: {}", next_position);
                                // and reinsert it if it's different
                                if next.key != current_key {
                                    // probably should swap back if this insert fails?
                                    return self.insert(next.key.clone(), next.value);
                                }
                            }
                        }
                        return Ok(());
                    }
                }
            } else {
                // eprintln!("now we have room at position: {}", position);
                // we got lucky and a slot is likely vacant now, but let's check
                let new = RawEntry::new(key.clone(), value);
                if let Some(current) = self.data[*position].swap(new) {
                    if current.key != key {
                        // probably should swap back if this insert fails?
                        return self.insert(current.key.clone(), current.value);
                    }
                }
                return Ok(());
            }
        }

        // our hash table is too full, but we can't do anything about that right
        // now. explode
        // TODO: return a real error
        Err((key, value))
    }

    pub fn get<Q: ?Sized + std::hash::Hash + Eq>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        for hash_id in 0..4 {
            let mut hasher = self.hashers[hash_id].build_hasher();
            key.hash(&mut hasher);
            let position = (hasher.finish() as usize) & (self.data.len() - 1);
            if let Some(entry) = self.data[position].load() {
                if key.eq(entry.key.borrow()) {
                    return Some(&entry.value);
                }
            }
        }
        None
    }

    pub fn remove<Q: ?Sized + std::hash::Hash + Eq>(&self, key: &Q)
    where
        K: Borrow<Q>,
    {
        for hash_id in 0..4 {
            let mut hasher = self.hashers[hash_id].build_hasher();
            key.hash(&mut hasher);
            let position = (hasher.finish() as usize) & (self.data.len() - 1);
            if let Some(entry) = self.data[position].load() {
                if key.eq(entry.key.borrow()) {
                    self.data[position].clear();
                }
            }
        }
    }
}

pub struct Iter<'a, K, V>
where
    K: std::cmp::PartialEq + std::hash::Hash + std::clone::Clone + std::fmt::Display,
{
    inner: &'a AtomicHashMap<K, V>,
    index: usize,
}

impl<'a, K, V> Iter<'a, K, V>
where
    K: std::cmp::PartialEq + std::hash::Hash + std::clone::Clone + std::fmt::Display,
{
    fn new(inner: &'a AtomicHashMap<K, V>) -> Iter<'a, K, V> {
        Iter { inner, index: 0 }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: std::cmp::PartialEq + std::hash::Hash + std::clone::Clone + std::fmt::Display,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        loop {
            if self.index < self.inner.data.len() {
                if let Some(entry) = self.inner.data[self.index].load() {
                    self.index += 1;
                    return Some((&entry.key, &entry.value));
                } else {
                    self.index += 1;
                }
            } else {
                return None;
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a AtomicHashMap<K, V>
where
    K: std::cmp::PartialEq + std::hash::Hash + std::clone::Clone + std::fmt::Display,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use rand::thread_rng;
    use rand_distr::*;
    use std::sync::Arc;

    #[test]
    fn basic() {
        let map = AtomicHashMap::<u64, u64>::with_capacity(128);
        assert!(map.get(&0).is_none());
        let _ = map.insert(1, 0);
        assert!(map.get(&0).is_none());
        let _ = map.insert(0, 0);
        assert_eq!(map.get(&0), Some(&0));
        let _ = map.insert(0, 1);
        assert_eq!(map.get(&0), Some(&1));
    }

    #[test]
    fn threaded() {
        let map = AtomicHashMap::<u64, u64>::with_capacity(128);
        assert!(map.get(&0).is_none());
        let map = Arc::new(map);
        let mut threads = Vec::new();
        let a = map.clone();
        threads.push(std::thread::spawn(move || {
            for _ in 0..1_000_000 {
                let _ = a.insert(0, 0);
            }
        }));
        let b = map.clone();
        threads.push(std::thread::spawn(move || {
            for _ in 0..1_000_000 {
                let _ = b.insert(0, 1);
            }
        }));
        for thread in threads {
            let _ = thread.join();
        }
        let value = map.get(&0);
        assert!(value == Some(&0) || value == Some(&1));
    }

    #[test]
    fn fill() {
        // table needs more room than you'd think
        let map = AtomicHashMap::<u8, u64>::with_capacity(u8::MAX as usize * 2);
        for k in 0..u8::MAX {
            let _ = map.insert(k, 0);
        }

        // table needs more room than you'd think
        let map = AtomicHashMap::<u16, u64>::with_capacity(u16::MAX as usize * 2);
        for k in 0..u16::MAX {
            let _ = map.insert(k, k.into());
            assert_eq!(map.get(&k), Some(&k.into()));
        }
        for k in 0..u16::MAX {
            assert_eq!(map.get(&k), Some(&k.into()));
        }
    }

    #[test]
    fn occupancy() {
        let capacity: u32 = 1_000_000;
        let map = AtomicHashMap::<u32, u64>::with_capacity(capacity as usize);
        let mut inserted = 0;
        for k in 0..capacity {
            if map.insert(k, 0).is_ok() {
                inserted += 1;
            }
        }
        let occupancy = 100.0 * inserted as f64 / capacity as f64;
        assert!(occupancy >= 95.0);
    }

    #[test]
    fn coherence() {
        let capacity: u32 = 1_000_000;
        let map = Arc::new(AtomicHashMap::<u32, u32>::with_capacity(capacity as usize));

        let mut threads = Vec::new();
        let a = map.clone();
        threads.push(std::thread::spawn(move || {
            let mut rng = thread_rng();
            let distribution = Uniform::new_inclusive(0.0, u32::MAX as f64);
            for _ in 0..10_000_000 {
                let value = distribution.sample(&mut rng).floor() as u32;
                let _ = a.insert(value, value);
            }
        }));
        let b = map.clone();
        threads.push(std::thread::spawn(move || {
            let mut rng = thread_rng();
            let distribution = Uniform::new_inclusive(0.0, u32::MAX as f64);
            for _ in 0..10_000_000 {
                let value = distribution.sample(&mut rng).floor() as u32;
                let _ = b.insert(value, value);
            }
        }));
        for thread in threads {
            let _ = thread.join();
        }

        for k in 0..capacity {
            let _ = map.insert(k, 0);
        }
        let mut counts = std::collections::HashMap::new();
        for entry in &map.data {
            if let Some(raw) = entry.load() {
                if !counts.contains_key(&raw.key) {
                    counts.insert(raw.key, 0);
                }
                if let Some(count) = counts.get_mut(&raw.key) {
                    *count += 1;
                }
            }
        }
        for (_key, value) in counts.iter() {
            assert_eq!(*value, 1);
        }
    }
}