use crate::minimizer::LexMinQueue;
use core::cmp::min;
use num_traits::int::PrimInt;

pub fn necklace_pos<const N: usize, T: PrimInt>(word: T) -> (T, usize) {
    let mut necklace = word;
    let mut rot = word;
    let mut pos = 0;
    for i in 1..N {
        rot = ((rot & T::one()) << (N - 1)) | (rot >> 1);
        if rot < necklace {
            necklace = rot;
            pos = i;
        }
    }
    (necklace, pos)
}

#[derive(Debug)]
pub struct NecklaceQueue<const N: usize, const W: usize, T: PrimInt> {
    word: T,
    min_queue: LexMinQueue<W, T>,
}

macro_rules! impl_necklace_queue {
($($T:ty),+) => {$(
impl<const N: usize, const W: usize> NecklaceQueue<N, W, $T> {
    const M: usize = N - W + 1;
    const MASK: $T = (1 << N) - 1;
    const MIN_MASK: $T = (1 << Self::M) - 1;

    pub fn new() -> Self {
        Self {
            word: 0,
            min_queue: LexMinQueue::new(),
        }
    }

    pub fn new_from_word(word: $T) -> Self {
        let mut res = Self::new();
        res.insert_full(word);
        res
    }

    #[inline]
    fn rotation(&self, p: usize) -> $T {
        ((self.word << p) & Self::MASK) | (self.word >> (N - p))
    }

    pub fn get_necklace_pos(&self) -> ($T, usize) {
        min(
            self.min_queue
                .iter_min_pos()
                .map(|p| (self.rotation(p), p))
                .min()
                .unwrap(),
            (W..N)
                .map(|p| (self.rotation(p), p))
                .min()
                .unwrap(),
        )
    }

    pub fn insert_full(&mut self, word: $T) {
        self.word = word & Self::MASK;
        let vals = (0..W).map(|p|
            (word >> (N - p - Self::M)) & Self::MIN_MASK
        );
        self.min_queue.insert_full(vals);
    }

    pub fn insert(&mut self, x: $T) {
        self.word = ((self.word << 1) & Self::MASK) | (x & 0b1);
        self.min_queue.insert(self.word & Self::MIN_MASK);
    }

    pub fn insert2(&mut self, x: $T) {
        self.word = ((self.word << 2) & Self::MASK) | (x & 0b11);
        self.min_queue.insert2((self.word >> 1) & Self::MIN_MASK, self.word & Self::MIN_MASK);
    }
}

impl<const N: usize, const W: usize> Default for NecklaceQueue<N, W, $T> {
    fn default() -> Self {
    Self::new()
    }
}
)*}}

impl_necklace_queue!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    const N: usize = 8;
    const M: usize = 5;
    const W: usize = N - M + 1;

    #[test]
    fn test_lex_min_queue_insert_full() {
        let mut min_queue = LexMinQueue::<W, _>::new();
        min_queue.insert_full([2, 1, 2, 1].iter());
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![W - 3, W - 1],
            "{:?}",
            min_queue
        );
    }

    #[test]
    fn test_lex_min_queue_insert() {
        let mut min_queue = LexMinQueue::<W, _>::new();
        min_queue.insert(3);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![W - 1],
            "{:?}",
            min_queue
        );
        min_queue.insert(1);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![W - 1],
            "{:?}",
            min_queue
        );
        min_queue.insert(2);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![W - 2],
            "{:?}",
            min_queue
        );
        min_queue.insert(3);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![W - 3],
            "{:?}",
            min_queue
        );
        min_queue.insert(1);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![W - 4, W - 1],
            "{:?}",
            min_queue
        );
        min_queue.insert(2);
        assert_eq!(
            min_queue.iter_min_pos().collect_vec(),
            vec![W - 2],
            "{:?}",
            min_queue
        );
    }

    #[test]
    fn test_necklace_queue() {
        let mut necklace_queue = NecklaceQueue::<N, W, u64>::new_from_word(0b10010110);
        assert_eq!(necklace_queue.get_necklace_pos(), (0b00101101, 1));
        necklace_queue.insert(0);
        assert_eq!(necklace_queue.get_necklace_pos(), (0b00001011, N - 2));
    }
}
