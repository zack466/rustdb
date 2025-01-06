use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use crate::value::Value;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Entry {
    key: String,
    value: Value,
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} -> {:?})", self.key, self.value)
    }
}

fn hash(key: &str, level: usize) -> usize {
    // Use FNV-1a hash algorithm
    let mut hash: u64 = 14695981039346656037; // FNV offset basis
    for byte in key.bytes() {
        hash = hash ^ (byte as u64);
        hash = hash.wrapping_mul(1099511628211); // FNV prime
    }

    // Use level to determine number of bits to use from hash
    hash as usize % (1 << level)
}

const OVERFLOW_SIZE: usize = 1;

// TODO: experience with fixed-size arrays (page + overflow) or other data structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Bucket {
    // We use a resizeable vector for chaining for simplicity's sake (and to avoid the horrors of using linked lists in Rust).
    entries: Vec<Entry>,
}

impl Bucket {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

// An in-memory hash table. Uses linear hashing with uncontrolled splitting.
// Owns all of its contents and can be serialized to disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Table {
    // The table consists of a vector of buckets, each containing multiple entries for chaining.
    // We also assume that the table has at least 2^current_level buckets.
    data: Vec<Bucket>,

    // The current level of linear hashing
    current_level: usize,

    // The next bucket to split (linear hashing)
    next: usize,
}

// TODO: try to implement LH* (distributed linear hashing)
impl Table {
    pub fn new() -> Self {
        Self {
            data: vec![Bucket::new()],
            current_level: 0,
            next: 0,
        }
    }

    fn index(&self, key: &str) -> usize {
        let index = hash(&key, self.current_level);

        // If index less than next, then the bucket has been split this round, so we
        // take the higher level hash function to get the right bucket (which may be the same one).
        if index < self.next {
            hash(&key, self.current_level + 1)
        } else {
            index
        }
    }

    pub fn set(&mut self, key: String, value: Value) {
        let index = self.index(&key);

        // First check if entry already exists, and modify it if so.
        for entry in self.data[index].entries.iter_mut() {
            if entry.key == key {
                entry.value = value;
                return;
            }
        }

        // Otherwise, add the entry to the bucket.
        self.data[index].entries.push(Entry { key, value });

        // If the bucket is full, split the next bucket (not necessarily this one)
        if self.data[index].entries.len() > OVERFLOW_SIZE {
            self.split();
        }
    }

    pub fn split(&mut self) {
        // Split the bucket at the next index
        self.data.push(Bucket::new());

        // Rehash entries from bucket being split
        let old_entries = self.data[self.next].entries.drain(..).collect::<Vec<_>>();
        for entry in old_entries {
            let index = hash(&entry.key, self.current_level + 1);
            self.data[index].entries.push(entry);
        }

        // Update next index and level
        self.next += 1;
        if self.next >= (1 << self.current_level) {
            self.current_level += 1;
            self.next = 0;
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let index = self.index(&key);

        for entry in self.data[index].entries.iter() {
            if entry.key == key {
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub fn to_disk(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;
        bincode::serialize_into(&mut file, &self)?;
        Ok(())
    }

    pub fn from_disk(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let table = bincode::deserialize_from(&mut file)?;
        Ok(table)
    }
}
