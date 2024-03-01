use super::minimizer::LexMinQueue;
use core::cmp::min;
use num_traits::int::PrimInt;

/// A data structure for computing necklaces of consecutive words.
///
/// It uses a monotone queue to keep track of the lexicographic minimizers.
///
/// # Type parameters
/// - `BITS`: the number of bits of the words.
/// - `T`: the integer type used to store the words.
/// - `WIDTH`: the width of the monotone queue (`BITS` - `M` + 1 for minimizers of `M` bits).
#[derive(Debug, Default)]
pub struct NecklaceQueue<
    const BITS: usize,
    T: PrimInt,
    const WIDTH: usize,
    const REVERSE: bool = false,
> {
    word: T,
    min_queue: LexMinQueue<WIDTH, T>,
}

macro_rules! impl_necklace_queue {
($($T:ty),+) => {$(
impl<const BITS: usize, const WIDTH: usize, const REVERSE: bool> NecklaceQueue<BITS, $T, WIDTH, REVERSE> {
    const M: usize = BITS - WIDTH + 1;
    const MASK: $T = (1 << BITS) - 1;
    const MIN_MASK: $T = (1 << Self::M) - 1;

    /// Creates an empty `NecklaceQueue`.
    pub fn new() -> Self {
        Self {
            word: 0,
            min_queue: LexMinQueue::new(),
        }
    }

    /// Creates a `NecklaceQueue` from a word.
    pub fn new_from_word(word: $T) -> Self {
        let mut res = Self::new();
        res.insert_full(word);
        res
    }

    /// Returns the cyclic rotation of the current word by `p` bits.
    #[inline]
    fn rotation(&self, p: usize) -> $T {
        ((self.word << p) & Self::MASK) | (self.word >> (BITS - p))
    }

    /// Returns the necklace of the current word and its position.
    pub fn get_necklace_pos(&self) -> ($T, usize) {
        if REVERSE {
            min(
                self.min_queue
                    .iter_min_pos()
                    .map(|p| (self.rotation(WIDTH - 1 - p), WIDTH - 1 - p))
                    .min()
                    .unwrap(),
                (WIDTH..BITS)
                    .map(|p| (self.rotation(p), p))
                    .min()
                    .unwrap(),
            )
        } else {
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
    }

    /// Inserts a whole word in the `NecklaceQueue`.
    pub fn insert_full(&mut self, word: $T) {
        if REVERSE {
            self.word = word & Self::MASK;
            let vals = (0..WIDTH).map(|p|
                (word >> p) & Self::MIN_MASK
            );
            self.min_queue.insert_full(vals);
        } else {
            self.word = word & Self::MASK;
            let vals = (0..WIDTH).map(|p|
                (word >> (BITS - p - Self::M)) & Self::MIN_MASK
            );
            self.min_queue.insert_full(vals);
        }
    }

    /// Inserts a bit in the `NecklaceQueue`.
    pub fn insert(&mut self, x: $T) {
        if REVERSE {
            self.word = (self.word >> 1) | ((x & 0b1) << (BITS - 1));
            self.min_queue.insert(self.word >> (WIDTH - 1));
        } else {
            self.word = ((self.word << 1) & Self::MASK) | (x & 0b1);
            self.min_queue.insert(self.word & Self::MIN_MASK);
        }
    }

    /// Inserts two bits in the `NecklaceQueue`.
    pub fn insert2(&mut self, x: $T) {
        if REVERSE {
            self.word = (self.word >> 2) | ((x & 0b11) << (BITS - 2));
            self.min_queue.insert2((self.word >> (WIDTH - 2)) & Self::MIN_MASK, self.word >> (WIDTH - 1));
        } else {
            self.word = ((self.word << 2) & Self::MASK) | (x & 0b11);
            self.min_queue.insert2((self.word >> 1) & Self::MIN_MASK, self.word & Self::MIN_MASK);
        }
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

    #[test]
    fn test_necklace_queue_rev() {
        let mut necklace_queue = NecklaceQueue::<BITS, u64, WIDTH, true>::new_from_word(0b10010110);
        assert_eq!(necklace_queue.get_necklace_pos(), (0b00101101, 1));
        necklace_queue.insert(1);
        assert_eq!(necklace_queue.get_necklace_pos(), (0b00101111, 2));
    }
}
