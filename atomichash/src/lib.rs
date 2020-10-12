//! Atomic HashMap
//!
//! Implements a hashmap datastructure by using atomic pointers for entries.
//! Unlike the standard hashmap or other concurrent hashmap implementations, the
//! hash table will not resize. This means that inserts may fail if the
//! occupancy is too high or a specific set of collisions occur.
//!
//! # Implementation Details
//!
//! The `AtomicHashMap` uses cuckoo hashing without any linear probing. In the
//! current implementation the same hash function is used with four distinct
//! initialization vectors which were chosen randomly at time of writing. This
//! results in each key residing in one of four entries if it is present in the
//! hashmap.
//!
//! When a new key is inserted, if all four entries are occupied, each of those
//! entries is examined to see if it can be moved to an empty entry based on the
//! hash functions for its key. This means we check up to 20 entries in the
//! worst case before returning an error.
//!
//! For read operations, each of the four possible entries are checked until a
//! match is found or no more entries exist to check.
//!
//! # Performance
//!
//! We aim to provide a high-performance implementation which prioritizes the
//! speed for accessing existing elements over the speed of inserting new
//! elements into the map. Given the interior mutability, this makes the map
//! suitable for workloads that read and write to a set of keys which is largely
//! fixed over the lifetime of the program. For datasets which have more churn
//! in terms of keys, it would be worth considering other concurrent hashmap
//! implementations.
//!
//! # Example
//!
//! Create a clocksource and read from it
//!
//! ```
//! use core::sync::atomic::{AtomicU64, Ordering};
//! use std::sync::Arc;
//!
//! use rustcommon_atomichash::AtomicHashMap;
//!
//! let map = Arc::new(AtomicHashMap::<u64, AtomicU64>::with_capacity(128));
//!
//! let mut threads = Vec::new();
//!
//! let a = map.clone();
//! threads.push(std::thread::spawn(move || {
//!     for _ in 0..1_000_000 {
//!         a.get(&0).store(0, Ordering::Relaxed);
//!     }
//! }));
//!
//! let b = map.clone();
//! threads.push(std::thread::spawn(move || {
//!     for _ in 0..1_000_000 {
//!         b.get(&0).store(1, Ordering::Relaxed);
//!     }
//! }));
//!
//! for thread in threads {
//!     let _ = thread.join();
//! }
//!
//! let value = map.get(&0).map(|v| v.load(Ordering::SeqCst));
//! // the last write is going to win
//! assert!(value == Some(0) || value == Some(1));
//! ```

mod entry;
mod hashmap;

pub use hashmap::AtomicHashMap;
