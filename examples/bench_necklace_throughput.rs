use cbl::necklace::NecklaceQueue;
use rand::{thread_rng, Rng};
use std::time::Instant;

// Loads runtime-provided constants for which declarations
// will be generated at `$OUT_DIR/constants.rs`.
pub mod constants {
    include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

use constants::{K, KT};

const N: usize = 10_000_000;
const BITS: usize = 2 * K;
const M: usize = 9;

fn main() {
    let mut rng = thread_rng();
    let mut bits = Vec::with_capacity(N);
    for _ in 0..N {
        bits.push(rng.gen());
    }
    let mut queue = NecklaceQueue::<BITS, KT, { BITS - M + 1 }>::new();
    queue.insert_full(rng.gen());
    let start = Instant::now();
    for x in bits {
        queue.insert(x);
        let (_necklace, _pos) = queue.get_necklace_pos();
    }
    let time = start.elapsed().as_nanos();
    eprintln!(
        "Computing a necklace of {BITS} bits from a stream takes {:.1} ns on average",
        time as f64 / N as f64
    );
}
