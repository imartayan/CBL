use crate::bitvector::{TinyBitvector, TinyBitvectorIterator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trie(Box<TrieNode>);

impl Trie {
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
    pub fn iter<const BYTES: usize>(&self) -> TrieIterator<BYTES> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrieNode {
    bv: TinyBitvector,
    children: Option<Vec<Trie>>,
}

impl TrieNode {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            bv: TinyBitvector::new(),
            children: None,
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.bv.is_empty()
    }

    pub fn count(&self) -> usize {
        if let Some(vec) = &self.children {
            vec.iter().map(|trie| trie.count()).sum()
        } else {
            self.bv.count()
        }
    }

    pub fn contains(&self, bytes: &[u8]) -> bool {
        assert!(!bytes.is_empty(), "The requested slice is empty");
        let index = bytes[0];
        let tail = &bytes[1..];
        if !self.bv.contains(index) {
            return false;
        }
        if self.children.is_none() {
            return tail.is_empty();
        }
        let rank = self.bv.rank(index);
        let vec = self.children.as_ref().unwrap();
        vec[rank].contains(tail)
    }

    pub fn insert(&mut self, bytes: &[u8]) -> bool {
        assert!(!bytes.is_empty(), "The requested slice is empty");
        let index = bytes[0];
        let tail = &bytes[1..];
        let absent = self.bv.insert(index);
        if tail.is_empty() {
            return absent;
        }
        let rank = self.bv.rank(index);
        let vec = self.children.get_or_insert(Vec::new());
        if absent {
            vec.insert(rank, Trie::new());
        }
        vec[rank].insert(tail)
    }

    pub fn remove(&mut self, bytes: &[u8]) -> bool {
        assert!(!bytes.is_empty(), "The requested slice is empty");
        let index = bytes[0];
        let tail = &bytes[1..];
        if !self.bv.contains(index) {
            return false;
        }
        if self.children.is_none() {
            self.bv.remove(index);
            return tail.is_empty();
        }
        let rank = self.bv.rank(index);
        let vec = self.children.as_mut().unwrap();
        let present = vec[rank].remove(tail);
        if vec[rank].is_empty() {
            self.bv.remove(index);
            vec.remove(rank);
        }
        if self.bv.is_empty() {
            self.children = None;
        }
        present
    }

    #[inline(always)]
    pub fn iter<const BYTES: usize>(&self) -> TrieIterator<BYTES> {
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
    trie: &'a TrieNode,
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
        if let Some(vec) = &self.trie.children {
            while self.tail.is_none() {
                self.index = self.index_iter.next();
                self.index?;
                self.rank = self.rank.map_or(Some(0), |rank| Some(rank + 1));
                let next_trie = &vec[self.rank.unwrap()];
                let mut tail_iter = Self {
                    trie: &next_trie.0,
                    depth: self.depth + 1,
                    index_iter: next_trie.0.bv.iter(),
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
        } else {
            self.index = self.index_iter.next();
            let mut bytes = [0u8; BYTES];
            bytes[self.depth] = self.index?;
            Some(bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie() {
        let mut trie = Trie::new();
        trie.insert(&[1, 2, 3]);
        trie.insert(&[1, 2, 4]);
        trie.insert(&[7, 7, 7]);
        assert!(!trie.is_empty());
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
        let mut trie = Trie::new();
        trie.insert(&[9, 9, 9]);
        trie.insert(&[1, 1, 1]);
        trie.insert(&[1, 2, 4]);
        trie.insert(&[1, 2, 3]);
        trie.insert(&[7, 7, 7]);
        let mut iter = trie.iter::<3>();
        assert_eq!(iter.next(), Some([1, 1, 1]));
        assert_eq!(iter.next(), Some([1, 2, 3]));
        assert_eq!(iter.next(), Some([1, 2, 4]));
        assert_eq!(iter.next(), Some([7, 7, 7]));
        assert_eq!(iter.next(), Some([9, 9, 9]));
        assert_eq!(iter.next(), None);
    }
}
