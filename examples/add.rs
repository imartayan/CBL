use cbl::kmer::*;
use cbl::reads::*;
use cbl::CBL;
use clap::Parser;
use core::cmp::min;
use std::time::Instant;

// Loads runtime-provided constants for which declarations
// will be generated at `$OUT_DIR/constants.rs`.
pub mod constants {
    include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

use constants::{K, KT};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (.fasta, .fa)
    input: String,
}

fn main() {
    let args = Args::parse();
    let input_filename = args.input.as_str();

    let mut cbl = CBL::<K, KT, u32>::new();
    let reads = Fasta::from_file(input_filename);

    reads.process_rec(|rec| {
        let seq = rec.seq();
        let chunk_size = 10000;
        let n = (seq.len() - K + 1) as u128;

        let now = Instant::now();
        for start in (0..(seq.len() - K + 1)).step_by(chunk_size) {
            cbl.insert_seq(&seq[start..(min(start + chunk_size + K - 1, seq.len()))]);
        }
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/insertion", elapsed / n);

        println!("prefix load: {}", cbl.prefix_load());
        println!("suffix load: {}", cbl.suffix_load());
        for (size, proportion) in cbl.suffix_load_repartition() {
            let percent = (proportion * 100.0).round();
            if percent > 0.0 {
                println!("size {size}: {percent} %");
            }
        }

        let mut _res = true;
        let now = Instant::now();
        for start in (0..(seq.len() - K + 1)).step_by(chunk_size) {
            for nucs in seq[start..(min(start + chunk_size + K - 1, seq.len()))].windows(K) {
                let kmer = RawKmer::<K, KT>::from_nucs(nucs);
                _res ^= cbl.contains(kmer);
            }
            // _res ^= cbl.contains_seq(&seq[start..(min(start + chunk_size + K - 1, seq.len()))]);
        }
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/membership", elapsed / n);

        let now = Instant::now();
        for start in (0..(seq.len() - K + 1)).step_by(chunk_size) {
            cbl.remove_seq(&seq[start..(min(start + chunk_size + K - 1, seq.len()))]);
        }
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/deletion", elapsed / n);
    });
}
