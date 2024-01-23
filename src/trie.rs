#![allow(dead_code)]

use crate::bitvector::{TinyBitvector, TinyBitvectorIterator};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trie<const BYTES: usize>(Box<TrieNode<BYTES>>);

impl<const BYTES: usize> Trie<BYTES> {
    #[inline(always)]
    pub fn new() -> Self {
        Self(Box::new(TrieNode::new()))
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        self.0.count()
    }

    #[inline(always)]
    pub fn contains(&self, bytes: &[u8]) -> bool {
        self.0.contains(bytes)
    }

    #[inline(always)]
    pub fn insert(&mut self, bytes: &[u8]) -> bool {
        self.0.insert(bytes)
    }

    #[inline(always)]
    pub fn remove(&mut self, bytes: &[u8]) -> bool {
        self.0.remove(bytes)
    }

    #[inline(always)]
    pub fn iter(&self) -> TrieIterator<BYTES> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrieNode<const BYTES: usize> {
    bv: TinyBitvector,
    children: Vec<Trie<BYTES>>,
}

impl<const BYTES: usize> TrieNode<BYTES> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            bv: TinyBitvector::new(),
            children: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.bv.is_empty()
    }

    pub fn count(&self) -> usize {
        let mut count = 0;
        let mut queue = VecDeque::new();
        queue.push_back(self);
        while !queue.is_empty() {
            let trie = queue.pop_front().unwrap();
            if trie.children.is_empty() {
                count += trie.bv.count();
            } else {
                for child in trie.children.iter() {
                    queue.push_back(&child.0);
                }
            }
        }
        count
    }

    pub fn contains(&self, bytes: &[u8]) -> bool {
        assert_eq!(bytes.len(), BYTES, "The trie takes slices of {BYTES} bytes");
        let mut trie = self;
        for &index in &bytes[..BYTES - 1] {
            if !trie.bv.contains(index) {
                return false;
            }
            let rank = trie.bv.rank(index);
            trie = &trie.children[rank].0;
        }
        let index = bytes[BYTES - 1];
        trie.bv.contains(index)
    }

    pub fn insert(&mut self, bytes: &[u8]) -> bool {
        assert_eq!(bytes.len(), BYTES, "The trie takes slices of {BYTES} bytes");
        let mut trie = self;
        for &index in &bytes[..BYTES - 1] {
            let absent = trie.bv.insert(index);
            let rank = trie.bv.rank(index);
            if absent {
                trie.children.insert(rank, Trie::new());
            }
            trie = &mut trie.children[rank].0;
        }
        let index = bytes[BYTES - 1];
        trie.bv.insert(index)
    }

    pub fn remove(&mut self, bytes: &[u8]) -> bool {
        assert_eq!(bytes.len(), BYTES, "The trie takes slices of {BYTES} bytes");
        let mut trie = self;
        let mut parents = Vec::new();
        for &index in &bytes[..BYTES - 1] {
            if !trie.bv.contains(index) {
                return false;
            }
            let rank = trie.bv.rank(index);
            parents.push(trie as *mut TrieNode<BYTES>);
            trie = &mut trie.children[rank].0;
        }
        let index = bytes[BYTES - 1];
        trie.bv.remove(index);
        if !trie.bv.is_empty() {
            return true;
        }
        for &index in bytes[..BYTES - 1].iter().rev() {
            unsafe {
                trie = parents.pop().unwrap().as_mut().unwrap();
            }
            let rank = trie.bv.rank(index);
            trie.children.remove(rank);
            trie.bv.remove(index);
            if !trie.bv.is_empty() {
                return true;
            }
        }
        true
    }

    #[inline(always)]
    pub fn iter(&self) -> TrieIterator<BYTES> {
        TrieIterator {
            trie: self,
            depth: 0,
            index_iter: self.bv.iter(),
            index: None,
            rank: None,
            tail_iter: None,
            tail: None,
        }
    }
}

pub struct TrieIterator<'a, const BYTES: usize> {
    trie: &'a TrieNode<BYTES>,
    depth: usize,
    index_iter: TinyBitvectorIterator<'a>,
    index: Option<u8>,
    rank: Option<usize>,
    tail_iter: Option<Box<TrieIterator<'a, BYTES>>>,
    tail: Option<[u8; BYTES]>,
}

impl<'a, const BYTES: usize> Iterator for TrieIterator<'a, BYTES> {
    type Item = [u8; BYTES];
    fn next(&mut self) -> Option<Self::Item> {
        if self.trie.children.is_empty() {
            self.index = self.index_iter.next();
            let mut bytes = [0u8; BYTES];
            bytes[self.depth] = self.index?;
            Some(bytes)
        } else {
            while self.tail.is_none() {
                self.index = self.index_iter.next();
                self.index?;
                self.rank = self.rank.map_or(Some(0), |rank| Some(rank + 1));
                let next_trie = &self.trie.children[self.rank.unwrap()].0;
                let mut tail_iter = Self {
                    trie: next_trie,
                    depth: self.depth + 1,
                    index_iter: next_trie.bv.iter(),
                    index: None,
                    rank: None,
                    tail_iter: None,
                    tail: None,
                };
                self.tail = tail_iter.next();
                self.tail_iter = Some(Box::new(tail_iter));
            }
            let mut bytes = self.tail?;
            bytes[self.depth] = self.index?;
            self.tail = self.tail_iter.as_mut().unwrap().next();
            Some(bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie() {
        let mut trie = Trie::<3>::new();
        trie.insert(&[1, 2, 3]);
        trie.insert(&[1, 2, 4]);
        trie.insert(&[7, 7, 7]);
        assert!(!trie.is_empty());
        assert_eq!(trie.count(), 3);
        assert!(trie.contains(&[1, 2, 3]));
        assert!(trie.contains(&[1, 2, 4]));
        assert!(trie.contains(&[7, 7, 7]));
        assert!(!trie.contains(&[1, 2, 1]));
        assert!(!trie.contains(&[3, 3, 3]));
        trie.remove(&[1, 2, 3]);
        trie.remove(&[1, 2, 4]);
        trie.remove(&[7, 7, 7]);
        assert!(trie.is_empty());
    }

    #[test]
    fn test_trie_iter() {
        let mut trie = Trie::<3>::new();
        trie.insert(&[9, 9, 9]);
        trie.insert(&[1, 1, 1]);
        trie.insert(&[1, 2, 4]);
        trie.insert(&[1, 2, 3]);
        trie.insert(&[7, 7, 7]);
        let mut iter = trie.iter();
        assert_eq!(iter.next(), Some([1, 1, 1]));
        assert_eq!(iter.next(), Some([1, 2, 3]));
        assert_eq!(iter.next(), Some([1, 2, 4]));
        assert_eq!(iter.next(), Some([7, 7, 7]));
        assert_eq!(iter.next(), Some([9, 9, 9]));
        assert_eq!(iter.next(), None);
    }
}
