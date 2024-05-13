use super::{TrieVec, WordSet};
use core::ops::*;
use iter_set_ops::*;
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    pub fn merge(mut wordsets: Vec<&mut Self>) -> Self {
        let mut res = Self::new();
        let ptr = wordsets.as_mut_ptr();
        let mut prefix_iters = wordsets
            .iter()
            .map(|set| set.prefixes.iter().enumerate())
            .collect_vec();
        for details in merge_iters_detailed_by(&mut prefix_iters, |(_, x), (_, y)| x.cmp(y)) {
            let prefix = (details[0].1).1;
            let mut suffix_iters = Vec::new();
            for (i, (rank, _)) in details {
                // each mutable reference is unique since each index is unique
                let set = unsafe { ptr.add(i).as_mut().unwrap_unchecked() };
                let id = set.tiered.get(rank) as usize;
                suffix_iters.push(set.suffix_containers[id].iter_sorted());
            }
            let mut container = TrieVec::new();
            container.insert_sorted_iter(merge_iters(&mut suffix_iters));
            let rank = res.suffix_containers.len();
            res.suffix_containers.push(container);
            res.tiered.insert(rank, rank as u32);
            res.prefixes.insert(prefix);
        }
        res
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    pub fn intersect(mut wordsets: Vec<&mut Self>) -> Self {
        let mut res = Self::new();
        let ptr = wordsets.as_mut_ptr();
        let mut prefix_iters = wordsets
            .iter()
            .map(|set| set.prefixes.iter().enumerate())
            .collect_vec();
        for details in intersect_iters_detailed_by(&mut prefix_iters, |(_, x), (_, y)| x.cmp(y)) {
            let prefix = details[0].1;
            let mut suffix_iters = Vec::new();
            for (i, &(rank, _)) in details.iter().enumerate() {
                // each mutable reference is unique since each index is unique
                let set = unsafe { ptr.add(i).as_mut().unwrap_unchecked() };
                let id = set.tiered.get(rank) as usize;
                suffix_iters.push(set.suffix_containers[id].iter_sorted());
            }
            let mut container = TrieVec::new();
            container.insert_sorted_iter(intersect_iters(&mut suffix_iters));
            if !container.is_empty() {
                let rank = res.suffix_containers.len();
                res.suffix_containers.push(container);
                res.tiered.insert(rank, rank as u32);
                res.prefixes.insert(prefix);
            }
        }
        res
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitOr<Self>
    for &mut WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    type Output = WordSet<PREFIX_BITS, SUFFIX_BITS>;

    fn bitor(self, other: Self) -> Self::Output {
        let mut res = Self::Output::new();
        self.prefixes
            .iter()
            .enumerate()
            .merge_join_by(other.prefixes.iter().enumerate(), |(_, a), (_, b)| a.cmp(b))
            .for_each(|c| match c {
                Left((rank_left, prefix)) => {
                    let id_left = self.tiered.get(rank_left) as usize;
                    let container = self.suffix_containers[id_left].clone();
                    let rank = res.suffix_containers.len();
                    res.suffix_containers.push(container);
                    res.tiered.insert(rank, rank as u32);
                    res.prefixes.insert(prefix);
                }
                Right((rank_right, prefix)) => {
                    let id_right = other.tiered.get(rank_right) as usize;
                    let container = other.suffix_containers[id_right].clone();
                    let rank = res.suffix_containers.len();
                    res.suffix_containers.push(container);
                    res.tiered.insert(rank, rank as u32);
                    res.prefixes.insert(prefix);
                }
                Both((rank_left, prefix), (rank_right, _)) => {
                    let id_left = self.tiered.get(rank_left) as usize;
                    let id_right = other.tiered.get(rank_right) as usize;
                    let container = &mut self.suffix_containers[id_left]
                        | &mut other.suffix_containers[id_right];
                    let rank = res.suffix_containers.len();
                    res.suffix_containers.push(container);
                    res.tiered.insert(rank, rank as u32);
                    res.prefixes.insert(prefix);
                }
            });
        res
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitOrAssign<&mut Self>
    for WordSet<PREFIX_BITS, SUFFIX_BITS>
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
                self.suffix_containers[id] |= &mut other.suffix_containers[other_id];
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
        self.prefixes |= &other.prefixes;
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitAnd<Self>
    for &mut WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    type Output = WordSet<PREFIX_BITS, SUFFIX_BITS>;

    fn bitand(self, other: Self) -> Self::Output {
        let mut res = Self::Output::new();
        self.prefixes
            .iter()
            .enumerate()
            .merge_join_by(other.prefixes.iter().enumerate(), |(_, a), (_, b)| a.cmp(b))
            .for_each(|c| match c {
                Left(_) => (),
                Right(_) => (),
                Both((rank_left, prefix), (rank_right, _)) => {
                    let id_left = self.tiered.get(rank_left) as usize;
                    let id_right = other.tiered.get(rank_right) as usize;
                    let container = &mut self.suffix_containers[id_left]
                        & &mut other.suffix_containers[id_right];
                    if !container.is_empty() {
                        let rank = res.suffix_containers.len();
                        res.suffix_containers.push(container);
                        res.tiered.insert(rank, rank as u32);
                        res.prefixes.insert(prefix);
                    }
                }
            });
        res
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitAndAssign<&mut Self>
    for WordSet<PREFIX_BITS, SUFFIX_BITS>
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
                self.suffix_containers[id] &= &mut other.suffix_containers[other_id];
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
        self.prefixes &= &other.prefixes;
        for prefix in empty_prefixes {
            self.prefixes.remove(prefix);
        }
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> Sub<Self>
    for &mut WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    type Output = WordSet<PREFIX_BITS, SUFFIX_BITS>;

    fn sub(self, other: Self) -> Self::Output {
        let mut res = Self::Output::new();
        self.prefixes
            .iter()
            .enumerate()
            .merge_join_by(other.prefixes.iter().enumerate(), |(_, a), (_, b)| a.cmp(b))
            .for_each(|c| match c {
                Left((rank_left, prefix)) => {
                    let id_left = self.tiered.get(rank_left) as usize;
                    let container = self.suffix_containers[id_left].clone();
                    let rank = res.suffix_containers.len();
                    res.suffix_containers.push(container);
                    res.tiered.insert(rank, rank as u32);
                    res.prefixes.insert(prefix);
                }
                Right(_) => (),
                Both((rank_left, prefix), (rank_right, _)) => {
                    let id_left = self.tiered.get(rank_left) as usize;
                    let id_right = other.tiered.get(rank_right) as usize;
                    let container = &mut self.suffix_containers[id_left]
                        - &mut other.suffix_containers[id_right];
                    if !container.is_empty() {
                        let rank = res.suffix_containers.len();
                        res.suffix_containers.push(container);
                        res.tiered.insert(rank, rank as u32);
                        res.prefixes.insert(prefix);
                    }
                }
            });
        res
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> SubAssign<&mut Self>
    for WordSet<PREFIX_BITS, SUFFIX_BITS>
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
                self.suffix_containers[id] -= &mut other.suffix_containers[other_id];
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
        self.prefixes -= &other.prefixes;
        for prefix in nonempty_prefixes {
            self.prefixes.insert(prefix);
        }
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitXor<Self>
    for &mut WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    type Output = WordSet<PREFIX_BITS, SUFFIX_BITS>;

    fn bitxor(self, other: Self) -> Self::Output {
        let mut res = Self::Output::new();
        self.prefixes
            .iter()
            .enumerate()
            .merge_join_by(other.prefixes.iter().enumerate(), |(_, a), (_, b)| a.cmp(b))
            .for_each(|c| match c {
                Left((rank_left, prefix)) => {
                    let id_left = self.tiered.get(rank_left) as usize;
                    let container = self.suffix_containers[id_left].clone();
                    let rank = res.suffix_containers.len();
                    res.suffix_containers.push(container);
                    res.tiered.insert(rank, rank as u32);
                    res.prefixes.insert(prefix);
                }
                Right((rank_right, prefix)) => {
                    let id_right = other.tiered.get(rank_right) as usize;
                    let container = other.suffix_containers[id_right].clone();
                    let rank = res.suffix_containers.len();
                    res.suffix_containers.push(container);
                    res.tiered.insert(rank, rank as u32);
                    res.prefixes.insert(prefix);
                }
                Both((rank_left, prefix), (rank_right, _)) => {
                    let id_left = self.tiered.get(rank_left) as usize;
                    let id_right = other.tiered.get(rank_right) as usize;
                    let container = &mut self.suffix_containers[id_left]
                        ^ &mut other.suffix_containers[id_right];
                    if !container.is_empty() {
                        let rank = res.suffix_containers.len();
                        res.suffix_containers.push(container);
                        res.tiered.insert(rank, rank as u32);
                        res.prefixes.insert(prefix);
                    }
                }
            });
        res
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> BitXorAssign<&mut Self>
    for WordSet<PREFIX_BITS, SUFFIX_BITS>
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
                self.suffix_containers[id] ^= &mut other.suffix_containers[other_id];
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

    const N: usize = 1_000_000;
    const PREFIX_BITS: usize = 16;
    const SUFFIX_BITS: usize = 8;

    #[test]
    fn test_union() {
        let mut set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let mut set2 = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            set.insert(i);
        }
        for &i in v1.iter() {
            set2.insert(i);
        }
        let res = &mut set | &mut set2;
        for &i in v0.iter() {
            assert!(res.contains(i), "false negative for {i}");
        }
        for &i in v1.iter() {
            assert!(res.contains(i), "false negative for {i}");
        }
        for &i in v2.iter() {
            assert!(!res.contains(i), "false positive for {i}");
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
        let mut set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let mut set2 = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
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
        let res = &mut set & &mut set2;
        for &i in v0.iter() {
            assert!(!res.contains(i), "false positive for {i}");
        }
        for &i in v1.iter() {
            assert!(res.contains(i), "false negative for {i}");
        }
        for &i in v2.iter() {
            assert!(!res.contains(i), "false positive for {i}");
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
        let mut set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let mut set2 = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
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
        let res = &mut set - &mut set2;
        for &i in v0.iter() {
            assert!(res.contains(i), "false negative for {i}");
        }
        for &i in v1.iter() {
            assert!(!res.contains(i), "false positive for {i}");
        }
        for &i in v2.iter() {
            assert!(!res.contains(i), "false positive for {i}");
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
        let mut set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let mut set2 = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
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
        let res = &mut set ^ &mut set2;
        for &i in v0.iter() {
            assert!(res.contains(i), "false negative for {i}");
        }
        for &i in v1.iter() {
            assert!(!res.contains(i), "false positive for {i}");
        }
        for &i in v2.iter() {
            assert!(res.contains(i), "false negative for {i}");
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

    #[test]
    fn test_multi_merge() {
        const C: usize = 10;
        let mut sets = vec![WordSet::<PREFIX_BITS, SUFFIX_BITS>::new(); C];
        for (i, set) in sets.iter_mut().enumerate() {
            let v = (i..(C * N)).step_by(C).collect_vec();
            set.insert_batch(&v);
        }
        let set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::merge(sets.iter_mut().collect());
        assert_eq!(
            set.iter::<usize>().collect_vec(),
            (0..(C * N)).collect_vec()
        );
    }

    #[test]
    fn test_multi_intersect() {
        const C: usize = 10;
        let mut sets = vec![WordSet::<PREFIX_BITS, SUFFIX_BITS>::new(); C];
        for (i, set) in sets.iter_mut().enumerate() {
            let v = (i..(C * N)).step_by(C).collect_vec();
            set.insert_batch(&v);
        }
        let set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::intersect(sets.iter_mut().collect());
        assert!(set.is_empty());
    }
}
