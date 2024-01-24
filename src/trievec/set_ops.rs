use super::{TrieOrVec, TrieVec};
use crate::trie::Trie;
use core::cmp::Ordering;
use core::ops::*;

impl<const BYTES: usize> BitOr<Self> for &mut TrieVec<BYTES> {
    type Output = TrieVec<BYTES>;

    fn bitor(self, other: Self) -> Self::Output {
        let mut trie = Trie::new();
        let mut len = 0usize;
        if let TrieOrVec::Vec(vec) = &mut self.0 {
            vec.sort_unstable();
        }
        if let TrieOrVec::Vec(vec) = &mut other.0 {
            vec.sort_unstable();
        }
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(&b) {
                Ordering::Less => {
                    trie.insert(&a.to_be_bytes());
                    len = len.saturating_add(1);
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    trie.insert(&b.to_be_bytes());
                    len = len.saturating_add(1);
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    trie.insert(&a.to_be_bytes());
                    len = len.saturating_add(1);
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        while let Some(a) = x {
            trie.insert(&a.to_be_bytes());
            len = len.saturating_add(1);
            x = self_iter.next();
        }
        while let Some(b) = y {
            trie.insert(&b.to_be_bytes());
            len = len.saturating_add(1);
            y = other_iter.next();
        }
        TrieVec(TrieOrVec::Trie(trie, len))
    }
}

impl<const BYTES: usize> BitOrAssign<&mut Self> for TrieVec<BYTES> {
    fn bitor_assign(&mut self, other: &mut Self) {
        if let TrieOrVec::Vec(vec) = &mut self.0 {
            vec.sort_unstable();
        }
        if let TrieOrVec::Vec(vec) = &mut other.0 {
            vec.sort_unstable();
        }
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        let mut insertions = Vec::new();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(&b) {
                Ordering::Less => {
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    insertions.push(b);
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        while let Some(b) = y {
            insertions.push(b);
            y = other_iter.next();
        }
        self.insert_sorted_iter(insertions.into_iter());
    }
}

impl<const BYTES: usize> BitAnd<Self> for &mut TrieVec<BYTES> {
    type Output = TrieVec<BYTES>;

    fn bitand(self, other: Self) -> Self::Output {
        let mut trie = Trie::new();
        let mut len = 0usize;
        if let TrieOrVec::Vec(vec) = &mut self.0 {
            vec.sort_unstable();
        }
        if let TrieOrVec::Vec(vec) = &mut other.0 {
            vec.sort_unstable();
        }
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(&b) {
                Ordering::Less => {
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    trie.insert(&a.to_be_bytes());
                    len = len.saturating_add(1);
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        TrieVec(TrieOrVec::Trie(trie, len))
    }
}

impl<const BYTES: usize> Sub<Self> for &mut TrieVec<BYTES> {
    type Output = TrieVec<BYTES>;

    fn sub(self, other: Self) -> Self::Output {
        let mut trie = Trie::new();
        let mut len = 0usize;
        if let TrieOrVec::Vec(vec) = &mut self.0 {
            vec.sort_unstable();
        }
        if let TrieOrVec::Vec(vec) = &mut other.0 {
            vec.sort_unstable();
        }
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(&b) {
                Ordering::Less => {
                    trie.insert(&a.to_be_bytes());
                    len = len.saturating_add(1);
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        while let Some(a) = x {
            trie.insert(&a.to_be_bytes());
            len = len.saturating_add(1);
            x = self_iter.next();
        }
        TrieVec(TrieOrVec::Trie(trie, len))
    }
}

impl<const BYTES: usize> BitXor<Self> for &mut TrieVec<BYTES> {
    type Output = TrieVec<BYTES>;

    fn bitxor(self, other: Self) -> Self::Output {
        let mut trie = Trie::new();
        let mut len = 0usize;
        if let TrieOrVec::Vec(vec) = &mut self.0 {
            vec.sort_unstable();
        }
        if let TrieOrVec::Vec(vec) = &mut other.0 {
            vec.sort_unstable();
        }
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(&b) {
                Ordering::Less => {
                    trie.insert(&a.to_be_bytes());
                    len = len.saturating_add(1);
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    trie.insert(&b.to_be_bytes());
                    len = len.saturating_add(1);
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        while let Some(a) = x {
            trie.insert(&a.to_be_bytes());
            len = len.saturating_add(1);
            x = self_iter.next();
        }
        while let Some(b) = y {
            trie.insert(&b.to_be_bytes());
            len = len.saturating_add(1);
            y = other_iter.next();
        }
        TrieVec(TrieOrVec::Trie(trie, len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sliced_int::SlicedInt;
    use itertools::Itertools;

    const N: usize = 1000;
    const BYTES: usize = 2;

    #[test]
    fn test_union() {
        let mut container = TrieVec::<BYTES>::new();
        let mut container2 = TrieVec::<BYTES>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            container.insert(SlicedInt::from_int(i));
        }
        for &i in v1.iter() {
            container2.insert(SlicedInt::from_int(i));
        }
        container = &mut container | &mut container2;
        for &i in v0.iter() {
            assert!(
                container.contains(&SlicedInt::from_int(i)),
                "false negative for {i}"
            );
        }
        for &i in v1.iter() {
            assert!(
                container.contains(&SlicedInt::from_int(i)),
                "false negative for {i}"
            );
        }
        for &i in v2.iter() {
            assert!(
                !container.contains(&SlicedInt::from_int(i)),
                "false positive for {i}"
            );
        }
    }

    #[test]
    fn test_intersection() {
        let mut container = TrieVec::<BYTES>::new();
        let mut container2 = TrieVec::<BYTES>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            container.insert(SlicedInt::from_int(i));
        }
        for &i in v1.iter() {
            container.insert(SlicedInt::from_int(i));
            container2.insert(SlicedInt::from_int(i));
        }
        for &i in v2.iter() {
            container2.insert(SlicedInt::from_int(i));
        }
        container = &mut container & &mut container2;
        for &i in v0.iter() {
            assert!(
                !container.contains(&SlicedInt::from_int(i)),
                "false positive for {i}"
            );
        }
        for &i in v1.iter() {
            assert!(
                container.contains(&SlicedInt::from_int(i)),
                "false negative for {i}"
            );
        }
        for &i in v2.iter() {
            assert!(
                !container.contains(&SlicedInt::from_int(i)),
                "false positive for {i}"
            );
        }
    }

    #[test]
    fn test_difference() {
        let mut container = TrieVec::<BYTES>::new();
        let mut container2 = TrieVec::<BYTES>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            container.insert(SlicedInt::from_int(i));
        }
        for &i in v1.iter() {
            container.insert(SlicedInt::from_int(i));
            container2.insert(SlicedInt::from_int(i));
        }
        for &i in v2.iter() {
            container2.insert(SlicedInt::from_int(i));
        }
        container = &mut container - &mut container2;
        for &i in v0.iter() {
            assert!(
                container.contains(&SlicedInt::from_int(i)),
                "false negative for {i}"
            );
        }
        for &i in v1.iter() {
            assert!(
                !container.contains(&SlicedInt::from_int(i)),
                "false positive for {i}"
            );
        }
        for &i in v2.iter() {
            assert!(
                !container.contains(&SlicedInt::from_int(i)),
                "false positive for {i}"
            );
        }
    }

    #[test]
    fn test_symmetric_difference() {
        let mut container = TrieVec::<BYTES>::new();
        let mut container2 = TrieVec::<BYTES>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            container.insert(SlicedInt::from_int(i));
        }
        for &i in v1.iter() {
            container.insert(SlicedInt::from_int(i));
            container2.insert(SlicedInt::from_int(i));
        }
        for &i in v2.iter() {
            container2.insert(SlicedInt::from_int(i));
        }
        container = &mut container ^ &mut container2;
        for &i in v0.iter() {
            assert!(
                container.contains(&SlicedInt::from_int(i)),
                "false negative for {i}"
            );
        }
        for &i in v1.iter() {
            assert!(
                !container.contains(&SlicedInt::from_int(i)),
                "false positive for {i}"
            );
        }
        for &i in v2.iter() {
            assert!(
                container.contains(&SlicedInt::from_int(i)),
                "false negative for {i}"
            );
        }
    }
}
