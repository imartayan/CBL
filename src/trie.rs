use crate::bitvector::TinyBitvector;

pub struct Trie {
    bv: TinyBitvector,
    children: Option<Vec<Trie>>,
}

impl Trie {
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

    pub fn contains(&self, bytes: &[u8]) -> bool {
        assert!(!bytes.is_empty(), "The requested slice is empty");
        let index = bytes[0];
        let next = &bytes[1..];
        if !self.bv.contains(index) {
            return false;
        }
        if self.children.is_none() {
            return next.is_empty();
        }
        let rank = self.bv.rank(index);
        let vec = self.children.as_ref().unwrap();
        vec[rank].contains(next)
    }

    pub fn insert(&mut self, bytes: &[u8]) -> bool {
        assert!(!bytes.is_empty(), "The requested slice is empty");
        let index = bytes[0];
        let next = &bytes[1..];
        let absent = self.bv.insert(index);
        if next.is_empty() {
            return absent;
        }
        let rank = self.bv.rank(index);
        let vec = self.children.get_or_insert(Vec::new());
        if absent {
            vec.insert(rank, Trie::new());
        }
        vec[rank].insert(next)
    }

    pub fn remove(&mut self, bytes: &[u8]) -> bool {
        assert!(!bytes.is_empty(), "The requested slice is empty");
        let index = bytes[0];
        let next = &bytes[1..];
        if !self.bv.contains(index) {
            return false;
        }
        if self.children.is_none() {
            self.bv.remove(index);
            return next.is_empty();
        }
        let rank = self.bv.rank(index);
        let vec = self.children.as_mut().unwrap();
        let present = vec[rank].remove(next);
        if vec[rank].is_empty() {
            self.bv.remove(index);
            vec.remove(rank);
        }
        if self.bv.is_empty() {
            self.children = None;
        }
        present
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
}
