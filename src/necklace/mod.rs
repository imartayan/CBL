//! Manipulate necklaces (smallest cyclic rotation of a word).
#![allow(dead_code)]

pub(crate) mod minimizer;
mod queue;
pub mod rank;

use num_traits::int::PrimInt;
pub use queue::NecklaceQueue;

/// Return the necklace (smallest cyclic rotation) of a word and its position.
pub fn necklace_pos<const BITS: usize, T: PrimInt>(word: T) -> (T, usize) {
    let mut necklace = word;
    let mut rot = word;
    let mut pos = 0;
    for i in (1..BITS).rev() {
        rot = ((rot & T::one()) << (BITS - 1)) | (rot >> 1);
        if rot < necklace {
            necklace = rot;
            pos = i;
        }
    }
    (necklace, pos)
}

/// Recover a word from its necklace and position.
#[inline]
pub fn revert_necklace_pos<const BITS: usize, T: PrimInt>(necklace: T, pos: usize) -> T {
    ((necklace << (BITS - pos)) & ((T::one() << BITS) - T::one())) | (necklace >> pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::Rng;

    const N: usize = 1_000_000;
    const BITS: usize = 31;
    type T = u32;
    const M: usize = 9;
    const WIDTH: usize = BITS - M + 1;

    #[test]
    fn test_necklace_revert() {
        let mut rng = thread_rng();
        for _ in 0..N {
            let word: T = rng.gen::<T>() >> 1;
            let (necklace, pos) = necklace_pos::<BITS, T>(word);
            assert_eq!(revert_necklace_pos::<BITS, T>(necklace, pos), word);
        }
    }

    #[test]
    fn test_same_necklace() {
        let mut rng = thread_rng();
        for _ in 0..N {
            let word: T = rng.gen::<T>() >> 1;
            let necklace_queue = NecklaceQueue::<BITS, T, WIDTH>::new_from_word(word);
            assert_eq!(
                necklace_pos::<BITS, T>(word),
                necklace_queue.get_necklace_pos()
            );
        }
    }
}
