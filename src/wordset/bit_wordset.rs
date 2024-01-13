use crate::bitvector::*;
use crate::compact_int::CompactInt;
use crate::container::*;
use crate::ffi::{TieredVec28, UniquePtr, WithinUniquePtr};
use core::ops::*;
use num_traits::cast::AsPrimitive;
use num_traits::sign::Unsigned;
use num_traits::PrimInt;
use std::collections::{btree_map::Entry::Vacant, BTreeMap};

pub struct BitWordSet<const PREFIX_BITS: usize, const SUFFIX_BITS: usize>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    prefixes: Bitvector,
    tiered: UniquePtr<TieredVec28>,
    suffix_containers: Vec<SemiSortedVec<CompactInt<{ SUFFIX_BITS.div_ceil(8) }>, 32>>,
    empty_containers: Vec<usize>,
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitWordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    const PREFIX_BITS: usize = PREFIX_BITS;
    const SUFFIX_BITS: usize = SUFFIX_BITS;

    pub fn new() -> Self {
        Self {
            prefixes: Bitvector::new_with_bitlength(Self::PREFIX_BITS),
            tiered: TieredVec28::new().within_unique_ptr(),
            suffix_containers: Vec::new(),
            empty_containers: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.suffix_containers
            .iter()
            .map(|container| container.len())
            .sum()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.prefixes.count() == 0
    }

    #[inline]
    pub fn split_prefix_suffix<T: PrimInt + Unsigned + AsPrimitive<usize>>(
        word: T,
    ) -> (usize, CompactInt<{ SUFFIX_BITS.div_ceil(8) }>) {
        let suffix_mask: T = (T::one() << Self::SUFFIX_BITS) - T::one();
        (
            (word >> Self::SUFFIX_BITS).as_(),
            CompactInt::<{ SUFFIX_BITS.div_ceil(8) }>::from_int(word & suffix_mask),
        )
    }

    #[inline]
    pub fn contains<T: PrimInt + Unsigned + AsPrimitive<usize>>(&self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        if !self.prefixes.contains(prefix) {
            return false;
        }
        let rank = self.prefixes.rank(prefix);
        let id = self.tiered.get(rank) as usize;
        self.suffix_containers[id].contains(&suffix)
    }

    pub fn insert<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        let mut absent = self.prefixes.insert(prefix);
        let rank = self.prefixes.rank(prefix);
        if absent {
            match self.empty_containers.pop() {
                Some(id) => {
                    self.suffix_containers[id].insert(suffix);
                    self.tiered.insert(rank, id as u32);
                }
                None => {
                    let id = self.suffix_containers.len();
                    self.suffix_containers.push(SemiSortedVec::<
                        CompactInt<{ SUFFIX_BITS.div_ceil(8) }>,
                        32,
                    >::new_with_one(suffix));
                    self.tiered.insert(rank, id as u32);
                }
            };
        } else {
            let id = self.tiered.get(rank) as usize;
            absent = self.suffix_containers[id].insert(suffix);
        }
        absent
    }

    pub fn remove<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        let mut present = self.prefixes.contains(prefix);
        if present {
            let rank = self.prefixes.rank(prefix);
            let id = self.tiered.get(rank) as usize;
            present = self.suffix_containers[id].remove(&suffix);
            if self.suffix_containers[id].is_empty() {
                self.empty_containers.push(id);
                self.tiered.remove(rank);
                self.prefixes.remove(prefix);
            }
        }
        present
    }

    pub fn contains_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(
        &mut self,
        words: &[T],
    ) -> Vec<bool> {
        let mut res = Vec::with_capacity(words.len());
        let prefixes_suffixes: Vec<_> = words
            .iter()
            .map(|&word| Self::split_prefix_suffix(word))
            .collect();
        for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
            let prefix = group[0].0;
            if !self.prefixes.contains(prefix) {
                res.resize(res.len() + group.len(), false);
                continue;
            }
            let rank = self.prefixes.rank(prefix);
            let id = self.tiered.get(rank) as usize;
            for (_, suffix) in group.iter() {
                res.push(self.suffix_containers[id].contains(suffix));
            }
        }
        res
    }

    pub fn insert_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, words: &[T]) {
        let prefixes_suffixes: Vec<_> = words
            .iter()
            .map(|&word| Self::split_prefix_suffix(word))
            .collect();
        for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
            let prefix = group[0].0;
            let absent = self.prefixes.insert(prefix);
            let rank = self.prefixes.rank(prefix);
            let id = if absent {
                match self.empty_containers.pop() {
                    Some(id) => {
                        self.tiered.insert(rank, id as u32);
                        id
                    }
                    None => {
                        let id = self.suffix_containers.len();
                        self.suffix_containers
                            .push(
                                SemiSortedVec::<CompactInt<{ SUFFIX_BITS.div_ceil(8) }>, 32>::new(),
                            );
                        self.tiered.insert(rank, id as u32);
                        id
                    }
                }
            } else {
                self.tiered.get(rank) as usize
            };
            if group.len() == 1 {
                self.suffix_containers[id].insert(group[0].1);
            } else {
                self.suffix_containers[id].insert_iter(group.iter().map(|&(_, suffix)| suffix));
            }
        }
    }

    pub fn remove_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, words: &[T]) {
        let prefixes_suffixes: Vec<_> = words
            .iter()
            .map(|&word| Self::split_prefix_suffix(word))
            .collect();
        for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
            let prefix = group[0].0;
            if self.prefixes.contains(prefix) {
                let rank = self.prefixes.rank(prefix);
                let id = self.tiered.get(rank) as usize;
                if group.len() == 1 {
                    self.suffix_containers[id].remove(&group[0].1);
                } else {
                    self.suffix_containers[id].remove_iter(group.iter().map(|&(_, suffix)| suffix));
                }
                if self.suffix_containers[id].is_empty() {
                    // self.suffix_containers[id] = SemiSortedVec::new();
                    self.empty_containers.push(id);
                    self.tiered.remove(rank);
                    self.prefixes.remove(prefix);
                }
            }
        }
    }

    #[inline]
    pub fn prefix_load(&self) -> f64 {
        self.tiered.len() as f64 / (1 << Self::PREFIX_BITS) as f64
    }

    pub fn suffix_load(&self) -> f64 {
        let total_size: usize = self
            .suffix_containers
            .iter()
            .map(|container| container.len())
            .sum();
        total_size as f64 / self.suffix_containers.len() as f64
    }

    pub fn suffix_load_repartition(&self) -> BTreeMap<usize, f64> {
        let mut size_count = BTreeMap::new();
        for size in self
            .suffix_containers
            .iter()
            .map(|container| container.len())
        {
            if let Vacant(e) = size_count.entry(size) {
                e.insert(1usize);
            } else {
                *size_count.get_mut(&size).unwrap() += 1;
            }
        }
        size_count
            .iter()
            .map(|(&k, &v)| (k, v as f64 / self.suffix_containers.len() as f64))
            .collect()
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> Default
    for BitWordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitOrAssign<&mut Self>
    for BitWordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn bitor_assign(&mut self, other: &mut Self) {
        let mut prefix_iter = self.prefixes.iter();
        let mut prefix = prefix_iter.next();
        let mut rank = 0;
        for (other_rank, other_prefix) in other.prefixes.iter().enumerate() {
            while prefix.is_some() && prefix.unwrap() < other_prefix {
                // keep container
                prefix = prefix_iter.next();
                rank += 1;
            }
            if prefix.is_some() && prefix.unwrap() == other_prefix {
                // merge containers
                let id = self.tiered.get(rank) as usize;
                let other_id = other.tiered.get(other_rank) as usize;
                self.suffix_containers[id] =
                    &mut self.suffix_containers[id] | &mut other.suffix_containers[other_id];
                prefix = prefix_iter.next();
                rank += 1;
            } else {
                // insert container
                let id = self.suffix_containers.len();
                let other_id = other.tiered.get(other_rank) as usize;
                self.suffix_containers
                    .push(other.suffix_containers[other_id].clone());
                self.tiered.insert(rank, id as u32);
                rank += 1;
            }
        }
        drop(prefix_iter);
        self.prefixes |= &other.prefixes;
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitAndAssign<&mut Self>
    for BitWordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn bitand_assign(&mut self, other: &mut Self) {
        let mut prefix_iter = self.prefixes.iter();
        let mut prefix = prefix_iter.next();
        let mut rank = 0;
        let mut empty_prefixes = Vec::new();
        for (other_rank, other_prefix) in other.prefixes.iter().enumerate() {
            while prefix.is_some() && prefix.unwrap() < other_prefix {
                // remove container
                let id = self.tiered.get(rank) as usize;
                self.empty_containers.push(id);
                self.tiered.remove(rank);
                prefix = prefix_iter.next();
            }
            if prefix.is_some() && prefix.unwrap() == other_prefix {
                // intersect containers
                let id = self.tiered.get(rank) as usize;
                let other_id = other.tiered.get(other_rank) as usize;
                self.suffix_containers[id] =
                    &mut self.suffix_containers[id] & &mut other.suffix_containers[other_id];
                if self.suffix_containers[id].is_empty() {
                    self.empty_containers.push(id);
                    self.tiered.remove(rank);
                    empty_prefixes.push(prefix.unwrap());
                } else {
                    rank += 1;
                }
                prefix = prefix_iter.next();
            }
        }
        drop(prefix_iter);
        self.prefixes &= &other.prefixes;
        for prefix in empty_prefixes {
            self.prefixes.remove(prefix);
        }
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> SubAssign<&mut Self>
    for BitWordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn sub_assign(&mut self, other: &mut Self) {
        let mut prefix_iter = self.prefixes.iter();
        let mut prefix = prefix_iter.next();
        let mut rank = 0;
        let mut nonempty_prefixes = Vec::new();
        for (other_rank, other_prefix) in other.prefixes.iter().enumerate() {
            while prefix.is_some() && prefix.unwrap() < other_prefix {
                // keep container
                prefix = prefix_iter.next();
                rank += 1;
            }
            if prefix.is_some() && prefix.unwrap() == other_prefix {
                // subtract containers
                let id = self.tiered.get(rank) as usize;
                let other_id = other.tiered.get(other_rank) as usize;
                self.suffix_containers[id] =
                    &mut self.suffix_containers[id] - &mut other.suffix_containers[other_id];
                if self.suffix_containers[id].is_empty() {
                    self.empty_containers.push(id);
                    self.tiered.remove(rank);
                } else {
                    nonempty_prefixes.push(prefix.unwrap());
                    rank += 1;
                }
                prefix = prefix_iter.next();
            }
        }
        drop(prefix_iter);
        self.prefixes -= &other.prefixes;
        for prefix in nonempty_prefixes {
            self.prefixes.insert(prefix);
        }
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitXorAssign<&mut Self>
    for BitWordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn bitxor_assign(&mut self, other: &mut Self) {
        let mut prefix_iter = self.prefixes.iter();
        let mut prefix = prefix_iter.next();
        let mut rank = 0;
        let mut nonempty_prefixes = Vec::new();
        for (other_rank, other_prefix) in other.prefixes.iter().enumerate() {
            while prefix.is_some() && prefix.unwrap() < other_prefix {
                // keep container
                prefix = prefix_iter.next();
                rank += 1;
            }
            if prefix.is_some() && prefix.unwrap() == other_prefix {
                // xor containers
                let id = self.tiered.get(rank) as usize;
                let other_id = other.tiered.get(other_rank) as usize;
                self.suffix_containers[id] =
                    &mut self.suffix_containers[id] ^ &mut other.suffix_containers[other_id];
                if self.suffix_containers[id].is_empty() {
                    self.empty_containers.push(id);
                    self.tiered.remove(rank);
                } else {
                    nonempty_prefixes.push(prefix.unwrap());
                    rank += 1;
                }
                prefix = prefix_iter.next();
            } else {
                // insert container
                let id = self.suffix_containers.len();
                let other_id = other.tiered.get(other_rank) as usize;
                self.suffix_containers
                    .push(other.suffix_containers[other_id].clone());
                self.tiered.insert(rank, id as u32);
                rank += 1;
            }
        }
        drop(prefix_iter);
        self.prefixes ^= &other.prefixes;
        for prefix in nonempty_prefixes {
            self.prefixes.insert(prefix);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    const N: usize = 1_000_000;
    const PREFIX_BITS: usize = 24;
    const SUFFIX_BITS: usize = 8;

    #[test]
    fn test_insert_contains_remove() {
        let mut positive: Vec<_> = (0..(2 * N)).step_by(2).collect();
        let mut negative: Vec<_> = (0..(2 * N)).skip(1).step_by(2).collect();
        let mut rng = thread_rng();
        positive.shuffle(&mut rng);
        negative.shuffle(&mut rng);
        let mut set = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        for &i in positive.iter() {
            set.insert(i);
        }
        for &i in positive.iter() {
            assert!(set.contains(i));
        }
        for &i in negative.iter() {
            assert!(!set.contains(i));
        }
        for &i in positive.iter() {
            set.remove(i);
        }
        for &i in positive.iter() {
            assert!(!set.contains(i));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_batch_operations() {
        let mut set = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let words: Vec<_> = (0..(2 * N)).step_by(2).collect();
        set.insert_batch(&words);
        assert!(set.contains_batch(&words).iter().all(|&b| b));
        for i in (0..(2 * N)).skip(1).step_by(2) {
            assert!(!set.contains(i));
        }
        set.remove_batch(&words);
        for i in (0..(2 * N)).step_by(2) {
            assert!(!set.contains(i));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_union() {
        let mut set = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let mut set2 = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            set.insert(i);
        }
        for &i in v1.iter() {
            set2.insert(i);
        }
        set |= &mut set2;
        for &i in v0.iter() {
            assert!(set.contains(i), "false negative for {i}");
        }
        for &i in v1.iter() {
            assert!(set.contains(i), "false negative for {i}");
        }
        for &i in v2.iter() {
            assert!(!set.contains(i), "false positive for {i}");
        }
    }

    #[test]
    fn test_intersection() {
        let mut set = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let mut set2 = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            set.insert(i);
        }
        for &i in v1.iter() {
            set.insert(i);
            set2.insert(i);
        }
        for &i in v2.iter() {
            set2.insert(i);
        }
        set &= &mut set2;
        for &i in v0.iter() {
            assert!(!set.contains(i), "false positive for {i}");
        }
        for &i in v1.iter() {
            assert!(set.contains(i), "false negative for {i}");
        }
        for &i in v2.iter() {
            assert!(!set.contains(i), "false positive for {i}");
        }
    }

    #[test]
    fn test_difference() {
        let mut set = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let mut set2 = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            set.insert(i);
        }
        for &i in v1.iter() {
            set.insert(i);
            set2.insert(i);
        }
        for &i in v2.iter() {
            set2.insert(i);
        }
        set -= &mut set2;
        for &i in v0.iter() {
            assert!(set.contains(i), "false negative for {i}");
        }
        for &i in v1.iter() {
            assert!(!set.contains(i), "false positive for {i}");
        }
        for &i in v2.iter() {
            assert!(!set.contains(i), "false positive for {i}");
        }
    }

    #[test]
    fn test_symmetric_difference() {
        let mut set = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let mut set2 = BitWordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            set.insert(i);
        }
        for &i in v1.iter() {
            set.insert(i);
            set2.insert(i);
        }
        for &i in v2.iter() {
            set2.insert(i);
        }
        set ^= &mut set2;
        for &i in v0.iter() {
            assert!(set.contains(i), "false negative for {i}");
        }
        for &i in v1.iter() {
            assert!(!set.contains(i), "false positive for {i}");
        }
        for &i in v2.iter() {
            assert!(set.contains(i), "false negative for {i}");
        }
    }
}
