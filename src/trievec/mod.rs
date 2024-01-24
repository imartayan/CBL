mod set_ops;

use crate::sliced_int::SlicedInt;
use crate::trie::{Trie, TrieIterator};
use core::slice::Iter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TrieOrVec<const BYTES: usize> {
    Vec(Vec<SlicedInt<BYTES>>),
    Trie(Trie<BYTES>, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrieVec<const BYTES: usize>(TrieOrVec<BYTES>);

impl<const BYTES: usize> TrieVec<BYTES> {
    #[inline]
    pub fn new() -> Self {
        Self(TrieOrVec::Vec(Vec::new()))
    }

    #[inline]
    pub fn new_with_one(x: SlicedInt<BYTES>) -> Self {
        Self(TrieOrVec::Vec(vec![x]))
    }

    #[inline]
    pub fn len(&self) -> usize {
        match &self.0 {
            TrieOrVec::Vec(vec) => vec.len(),
            TrieOrVec::Trie(_, len) => *len,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match &self.0 {
            TrieOrVec::Vec(vec) => vec.is_empty(),
            TrieOrVec::Trie(_, len) => *len == 0,
        }
    }

    #[inline]
    pub fn contains(&self, x: &SlicedInt<BYTES>) -> bool {
        match &self.0 {
            TrieOrVec::Vec(vec) => vec.contains(x),
            TrieOrVec::Trie(trie, _) => trie.contains(&x.to_be_bytes()),
        }
    }

    pub fn insert(&mut self, x: SlicedInt<BYTES>) -> bool {
        match &mut self.0 {
            TrieOrVec::Trie(trie, len) => {
                let absent = trie.insert(&x.to_be_bytes());
                if absent {
                    *len += 1;
                }
                absent
            }
            TrieOrVec::Vec(vec) => {
                if !vec.contains(&x) {
                    vec.push(x);
                    return true;
                }
                false
            }
        }
    }

    pub fn remove(&mut self, x: &SlicedInt<BYTES>) -> bool {
        match &mut self.0 {
            TrieOrVec::Trie(trie, len) => {
                let present = trie.remove(&x.to_be_bytes());
                if present {
                    *len -= 1;
                }
                present
            }
            TrieOrVec::Vec(vec) => {
                if let Some(i) = vec.iter().position(|y| y == x) {
                    vec.swap_remove(i);
                    return true;
                }
                false
            }
        }
    }

    #[inline]
    pub fn insert_iter<I: Iterator<Item = SlicedInt<BYTES>>>(&mut self, it: I) {
        for x in it {
            self.insert(x);
        }
    }

    #[inline]
    fn insert_sorted_iter<I: Iterator<Item = SlicedInt<BYTES>>>(&mut self, it: I) {
        match &mut self.0 {
            TrieOrVec::Vec(vec) => {
                let stop = vec.len();
                let mut i = 0;
                for x in it {
                    while i < stop && x > vec[i] {
                        i += 1;
                    }
                    if i == stop || x < vec[i] {
                        vec.push(x);
                    }
                }
            }
            TrieOrVec::Trie(_trie, _len) => {
                self.insert_iter(it);
            }
        }
    }

    #[inline]
    pub fn remove_iter<I: Iterator<Item = SlicedInt<BYTES>>>(&mut self, it: I) {
        for x in it {
            self.remove(&x);
        }
    }

    pub fn as_trie(&mut self) {
        if let TrieOrVec::Vec(vec) = &self.0 {
            let mut trie = Trie::new();
            for x in vec.iter() {
                trie.insert(&x.to_be_bytes());
            }
            self.0 = TrieOrVec::Trie(trie, vec.len());
        }
    }

    pub fn as_vec(&mut self) {
        if let TrieOrVec::Trie(trie, _) = &self.0 {
            let vec = trie
                .iter()
                .map(|bytes: [u8; BYTES]| SlicedInt::from_be_bytes(&bytes))
                .collect();
            self.0 = TrieOrVec::Vec(vec);
        }
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> TrieVecIterator<'a, BYTES>
    where
        SlicedInt<BYTES>: 'a,
    {
        match &self.0 {
            TrieOrVec::Vec(vec) => TrieVecIterator::Vec(vec.iter()),
            TrieOrVec::Trie(trie, _) => TrieVecIterator::Trie(trie.iter()),
        }
    }
}

impl<const BYTES: usize> Default for TrieVec<BYTES> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum TrieVecIterator<'a, const BYTES: usize> {
    Vec(Iter<'a, SlicedInt<BYTES>>),
    Trie(TrieIterator<'a, BYTES>),
}

impl<'a, const BYTES: usize> Iterator for TrieVecIterator<'a, BYTES> {
    type Item = SlicedInt<BYTES>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Vec(iter) => iter.next().copied(),
            Self::Trie(iter) => iter.next().map(|bytes| SlicedInt::from_be_bytes(&bytes)),
        }
    }
}
