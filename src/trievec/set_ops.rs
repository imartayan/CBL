use super::{TrieOrVec, TrieVec};
use crate::trie::Trie;
use core::cmp::Ordering;
use core::ops::*;

impl<const BYTES: usize> BitOr<Self> for &mut TrieVec<BYTES> {
    type Output = TrieVec<BYTES>;

    fn bitor(self, other: Self) -> Self::Output {
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
                    insertions.push(a);
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    insertions.push(b);
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    insertions.push(a);
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        while let Some(a) = x {
            insertions.push(a);
            x = self_iter.next();
        }
        while let Some(b) = y {
            insertions.push(b);
            y = other_iter.next();
        }
        TrieVec(TrieOrVec::Vec(insertions))
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
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    insertions.push(a);
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        TrieVec(TrieOrVec::Vec(insertions))
    }
}

impl<const BYTES: usize> BitAndAssign<&mut Self> for TrieVec<BYTES> {
    fn bitand_assign(&mut self, other: &mut Self) {
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
        let mut deletions = Vec::new();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(&b) {
                Ordering::Less => {
                    deletions.push(a);
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
            deletions.push(a);
            x = self_iter.next();
        }
        self.remove_sorted_iter(deletions.into_iter());
    }
}

impl<const BYTES: usize> Sub<Self> for &mut TrieVec<BYTES> {
    type Output = TrieVec<BYTES>;

    fn sub(self, other: Self) -> Self::Output {
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
                    insertions.push(a);
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
            insertions.push(a);
            x = self_iter.next();
        }
        TrieVec(TrieOrVec::Vec(insertions))
    }
}

impl<const BYTES: usize> SubAssign<&mut Self> for TrieVec<BYTES> {
    fn sub_assign(&mut self, other: &mut Self) {
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
        let mut deletions = Vec::new();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(&b) {
                Ordering::Less => {
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    deletions.push(a);
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        self.remove_sorted_iter(deletions.into_iter());
    }
}

impl<const BYTES: usize> BitXor<Self> for &mut TrieVec<BYTES> {
    type Output = TrieVec<BYTES>;

    fn bitxor(self, other: Self) -> Self::Output {
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
                    insertions.push(a);
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
        while let Some(a) = x {
            insertions.push(a);
            x = self_iter.next();
        }
        while let Some(b) = y {
            insertions.push(b);
            y = other_iter.next();
        }
        TrieVec(TrieOrVec::Vec(insertions))
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
