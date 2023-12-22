use cbl::cbl::*;
use cbl::kmer::*;
use cbl::reads::*;
use clap::Parser;
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

    let mut cbl = CBL::<K, KT>::new();
    let reads = Fasta::from_file(input_filename);

    reads.process_rec(|rec| {
        let seq = rec.seq();
        let n = (seq.len() - K + 1) as u128;

        let now = Instant::now();
        cbl.insert_seq(seq);
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

        let now = Instant::now();
        for (i, kmer) in RawKmer::iter_from_nucs(seq.iter()).enumerate() {
            assert!(
                cbl.contains(kmer),
                "kmer {i} false negative: {:0b}",
                kmer.to_int()
            );
        }
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/membership", elapsed / n);

        let now = Instant::now();
        cbl.remove_seq(seq);
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/deletion", elapsed / n);
    });
}
