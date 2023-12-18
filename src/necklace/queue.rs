use super::minimizer::LexMinQueue;
use core::cmp::min;
use num_traits::int::PrimInt;

pub fn necklace_pos<const BITS: usize, T: PrimInt>(word: T) -> (T, usize) {
    let mut necklace = word;
    let mut rot = word;
    let mut pos = 0;
    for i in 1..BITS {
        rot = ((rot & T::one()) << (BITS - 1)) | (rot >> 1);
        if rot < necklace {
            necklace = rot;
            pos = i;
        }
    }
    (necklace, pos)
}

#[derive(Debug)]
pub struct NecklaceQueue<const BITS: usize, T: PrimInt, const WIDTH: usize> {
    word: T,
    min_queue: LexMinQueue<WIDTH, T>,
}

macro_rules! impl_necklace_queue {
($($T:ty),+) => {$(
impl<const BITS: usize, const WIDTH: usize> NecklaceQueue<BITS, $T, WIDTH> {
    const M: usize = BITS - WIDTH + 1;
    const MASK: $T = (1 << BITS) - 1;
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
        ((self.word << p) & Self::MASK) | (self.word >> (BITS - p))
    }

    pub fn get_necklace_pos(&self) -> ($T, usize) {
        min(
            self.min_queue
                .iter_min_pos()
                .map(|p| (self.rotation(p), p))
                .min()
                .unwrap(),
            (WIDTH..BITS)
                .map(|p| (self.rotation(p), p))
                .min()
                .unwrap(),
        )
    }

    pub fn insert_full(&mut self, word: $T) {
        self.word = word & Self::MASK;
        let vals = (0..WIDTH).map(|p|
            (word >> (BITS - p - Self::M)) & Self::MIN_MASK
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

impl<const BITS: usize, const WIDTH: usize> Default for NecklaceQueue<BITS, $T, WIDTH> {
    fn default() -> Self {
    Self::new()
    }
}
)*}}

impl_necklace_queue!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    use super::*;

    const BITS: usize = 8;
    const M: usize = 5;
    const WIDTH: usize = BITS - M + 1;

    #[test]
    fn test_necklace_queue() {
        let mut necklace_queue = NecklaceQueue::<BITS, u64, WIDTH>::new_from_word(0b10010110);
        assert_eq!(necklace_queue.get_necklace_pos(), (0b00101101, 1));
        necklace_queue.insert(0);
        assert_eq!(necklace_queue.get_necklace_pos(), (0b00001011, BITS - 2));
    }
}