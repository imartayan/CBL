use core::fmt::{Binary, Display};
use core::hash::Hash;
use core::iter::FilterMap;
use num_traits::cast::AsPrimitive;
use num_traits::int::PrimInt;
use num_traits::sign::Unsigned;

pub trait Base: PrimInt + Unsigned + AsPrimitive<usize> + Display + Binary {
    const BASE_MASK: Self;
    fn from_nuc(b: &u8) -> Option<Self>;
    fn to_nuc(self) -> u8;
    fn bases() -> [Self; 4];
}

pub trait RevComp {
    fn rev_comp(self) -> Self;
}

pub trait Kmer<const K: usize, T: Base>: Sized + Copy + RevComp + Ord + Hash {
    const MASK: T;
    fn from_int(s: T) -> Self;
    fn to_int(self) -> T;
    #[inline]
    fn new() -> Self {
        Self::from_int(T::zero())
    }
    #[inline]
    fn extend(self, base: T) -> Self {
        Self::from_int((self.to_int() << 2) | base)
    }
    #[inline]
    fn append(self, base: T) -> Self {
        Self::from_int(((self.to_int() << 2) | base) & Self::MASK)
    }
    #[inline]
    fn prepend(self, base: T) -> Self {
        Self::from_int((self.to_int() >> 2) | (base << (2 * (K - 1))))
    }
    #[inline]
    fn successors(self) -> [Self; 4] {
        T::bases().map(|base| self.append(base))
    }
    #[inline]
    fn predecessors(self) -> [Self; 4] {
        T::bases().map(|base| self.prepend(base))
    }
    #[inline]
    fn is_canonical(self) -> bool {
        self.to_int().count_ones() % 2 == 0
    }
    #[inline]
    fn canonical(self) -> Self {
        if self.is_canonical() {
            self
        } else {
            self.rev_comp()
        }
    }
    #[inline]
    fn from_bases_iter<I: Iterator<Item = T>>(bases: I) -> Self {
        bases.take(K).fold(Self::new(), |s, base| s.extend(base))
    }
    #[inline]
    fn from_bases(bases: &[T]) -> Self {
        Self::from_bases_iter(bases.iter().copied())
    }
    fn to_bases(self) -> [T; K] {
        let mut res = [T::zero(); K];
        let mut s = self.to_int();
        for i in 0..K {
            res[K - 1 - i] = s & T::BASE_MASK;
            s = s >> 2;
        }
        res
    }
    #[inline]
    fn from_nucs(nucs: &[u8]) -> Self {
        Self::from_bases_iter(nucs.iter().filter_map(T::from_nuc))
    }
    #[inline]
    fn to_nucs(self) -> [u8; K] {
        self.to_bases().map(|base| base.to_nuc())
    }
    fn iter_from_bases<I: Iterator<Item = T>>(bases: I) -> KmerIterator<K, T, Self, I> {
        KmerIterator {
            kmer: Self::new(),
            bases,
            init: false,
        }
    }
    #[inline]
    #[allow(clippy::type_complexity)]
    fn iter_from_nucs<'a, I: Iterator<Item = &'a u8>>(
        nucs: I,
    ) -> KmerIterator<K, T, Self, FilterMap<I, fn(&u8) -> Option<T>>> {
        Self::iter_from_bases(nucs.filter_map(T::from_nuc))
    }
}

pub struct KmerIterator<const K: usize, T, KT, I>
where
    T: Base,
    KT: Kmer<K, T>,
    I: Iterator<Item = T>,
{
    kmer: KT,
    bases: I,
    init: bool,
}

impl<const K: usize, T, KT, I> Iterator for KmerIterator<K, T, KT, I>
where
    T: Base,
    KT: Kmer<K, T>,
    I: Iterator<Item = T>,
{
    type Item = KT;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.init {
            self.init = true;
            for _ in 0..K {
                self.kmer = self.kmer.extend(self.bases.next()?);
            }
            Some(self.kmer)
        } else {
            self.kmer = self.kmer.append(self.bases.next()?);
            Some(self.kmer)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct IntKmer<const K: usize, T: Base>(T);

macro_rules! impl_t {
($($T:ty),+) => {$(
    impl Base for $T {
        const BASE_MASK: Self = 0b11;
        #[inline]
        fn from_nuc(b: &u8) -> Option<Self> {
            match b {
                b'A' | b'C' | b'G' | b'T' => Some(((b / 2) % 4) as $T),
                _ => None,
            }
        }
        #[inline]
        fn to_nuc(self) -> u8 {
            const BASE_LOOKUP: [u8; 4] = [b'A', b'C', b'T', b'G'];
            debug_assert!(self < 4, "Invalid base");
            BASE_LOOKUP[self as usize]
        }
        #[inline(always)]
        fn bases() -> [Self; 4] {
            [0, 1, 2, 3]
        }
    }

    impl<const K: usize> Kmer<K, $T> for IntKmer<K, $T> {
        const MASK: $T = (1 << (2 * K)) - 1;
        #[inline(always)]
        fn from_int(s: $T) -> Self {
            Self(s)
        }
        #[inline(always)]
        fn to_int(self) -> $T {
            self.0
        }
    }
)*}}

impl_t!(u8, u16, u32, u64, u128);

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
impl<const K: usize> RevComp for IntKmer<K, u8> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().reverse_bits();
        res = (res >> 1 & 0x55) | (res & 0x55) << 1;
        res ^= 0xAA;
        Self::from_int(res >> (2 * (4 - K)))
    }
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
impl<const K: usize> RevComp for IntKmer<K, u16> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().reverse_bits();
        res = (res >> 1 & 0x5555) | (res & 0x5555) << 1;
        res ^= 0xAAAA;
        Self::from_int(res >> (2 * (8 - K)))
    }
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
impl<const K: usize> RevComp for IntKmer<K, u32> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().reverse_bits();
        res = (res >> 1 & 0x5555_5555) | (res & 0x5555_5555) << 1;
        res ^= 0xAAAA_AAAA;
        Self::from_int(res >> (2 * (16 - K)))
    }
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
impl<const K: usize> RevComp for IntKmer<K, u64> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().reverse_bits();
        res = (res >> 1 & 0x5555_5555_5555_5555) | (res & 0x5555_5555_5555_5555) << 1;
        res ^= 0xAAAA_AAAA_AAAA_AAAA;
        Self::from_int(res >> (2 * (32 - K)))
    }
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
impl<const K: usize> RevComp for IntKmer<K, u128> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().reverse_bits();
        res = (res >> 1 & 0x5555_5555_5555_5555_5555_5555_5555_5555)
            | (res & 0x5555_5555_5555_5555_5555_5555_5555_5555) << 1;
        res ^= 0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA;
        Self::from_int(res >> (2 * (64 - K)))
    }
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
impl<const K: usize> RevComp for IntKmer<K, u8> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int();
        res = (res >> 4 & 0x0F) | (res & 0x0F) << 4;
        res = (res >> 2 & 0x33) | (res & 0x33) << 2;
        res ^= 0xAA;
        Self::from_int(res >> (2 * (4 - K)))
    }
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
impl<const K: usize> RevComp for IntKmer<K, u16> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().swap_bytes();
        res = (res >> 4 & 0x0F0F) | (res & 0x0F0F) << 4;
        res = (res >> 2 & 0x3333) | (res & 0x3333) << 2;
        res ^= 0xAAAA;
        Self::from_int(res >> (2 * (8 - K)))
    }
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
impl<const K: usize> RevComp for IntKmer<K, u32> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().swap_bytes();
        res = (res >> 4 & 0x0F0F_0F0F) | (res & 0x0F0F_0F0F) << 4;
        res = (res >> 2 & 0x3333_3333) | (res & 0x3333_3333) << 2;
        res ^= 0xAAAA_AAAA;
        Self::from_int(res >> (2 * (16 - K)))
    }
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
impl<const K: usize> RevComp for IntKmer<K, u64> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().swap_bytes();
        res = (res >> 4 & 0x0F0F_0F0F_0F0F_0F0F) | (res & 0x0F0F_0F0F_0F0F_0F0F) << 4;
        res = (res >> 2 & 0x3333_3333_3333_3333) | (res & 0x3333_3333_3333_3333) << 2;
        res ^= 0xAAAA_AAAA_AAAA_AAAA;
        Self::from_int(res >> (2 * (32 - K)))
    }
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
impl<const K: usize> RevComp for IntKmer<K, u128> {
    fn rev_comp(self) -> Self {
        let mut res = self.to_int().swap_bytes();
        res = (res >> 4 & 0x0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F)
            | (res & 0x0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F) << 4;
        res = (res >> 2 & 0x3333_3333_3333_3333_3333_3333_3333_3333)
            | (res & 0x3333_3333_3333_3333_3333_3333_3333_3333) << 2;
        res ^= 0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA;
        Self::from_int(res >> (2 * (64 - K)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc_8() {
        let kmer = IntKmer::<4, u8>::from_nucs(b"ATCG");
        assert_eq!(kmer.rev_comp().to_nucs(), *b"CGAT");
    }
    #[test]
    fn test_rc_16() {
        let kmer = IntKmer::<4, u16>::from_nucs(b"ATCG");
        assert_eq!(kmer.rev_comp().to_nucs(), *b"CGAT");
    }
    #[test]
    fn test_rc_32() {
        let kmer = IntKmer::<11, u32>::from_nucs(b"CATAATCCAGC");
        assert_eq!(kmer.rev_comp().to_nucs(), *b"GCTGGATTATG");
    }
    #[test]
    fn test_rc_64() {
        let kmer = IntKmer::<11, u64>::from_nucs(b"CATAATCCAGC");
        assert_eq!(kmer.rev_comp().to_nucs(), *b"GCTGGATTATG");
    }
    #[test]
    fn test_rc_128() {
        let kmer = IntKmer::<11, u128>::from_nucs(b"CATAATCCAGC");
        assert_eq!(kmer.rev_comp().to_nucs(), *b"GCTGGATTATG");
    }
    #[test]
    fn rc_rc_8() {
        for i in 0..64 {
            let kmer = IntKmer::<3, u8>::from_int(i);
            assert_eq!(kmer.rev_comp().rev_comp().to_int(), i);
        }
    }
    #[test]
    fn rc_rc_16() {
        for i in 0..16384 {
            let kmer = IntKmer::<7, u16>::from_int(i);
            assert_eq!(kmer.rev_comp().rev_comp().to_int(), i);
        }
    }
    #[test]
    fn rc_rc_32() {
        for i in 0..1_000_000 {
            let kmer = IntKmer::<15, u32>::from_int(i);
            assert_eq!(kmer.rev_comp().rev_comp().to_int(), i);
        }
    }
    #[test]
    fn rc_rc_64() {
        for i in 0..1_000_000 {
            let kmer = IntKmer::<15, u64>::from_int(i);
            assert_eq!(kmer.rev_comp().rev_comp().to_int(), i);
        }
    }
    #[test]
    fn rc_rc_128() {
        for i in 0..1_000_000 {
            let kmer = IntKmer::<15, u128>::from_int(i);
            assert_eq!(kmer.rev_comp().rev_comp().to_int(), i);
        }
    }
}
