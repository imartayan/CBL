use super::WordSet;
use core::ops::*;

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
    const PREFIX_BITS: usize = 24;
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
