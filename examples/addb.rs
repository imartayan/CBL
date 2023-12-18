use cbl::kmer::{Base, Kmer, RawKmer};
use cbl::reads::*;
use clap::Parser;
use std::collections::BTreeSet;
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
    let reads = Fasta::from_file(input_filename);
    let mut set = BTreeSet::new();

    reads.process_rec(|rec| {
        let seq = rec.seq();
        let n = (seq.len() - K + 1) as u128;
        let mut kmer = RawKmer::<K, KT>::new();

        let now = Instant::now();
        for (i, base) in seq.iter().filter_map(KT::from_nuc).enumerate() {
            if i < K - 1 {
                kmer = kmer.extend(base as KT);
            } else {
                kmer = kmer.append(base as KT);
                set.insert(kmer);
            }
        }
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/insertion", elapsed / n);

        println!("{} kmers", set.len());

        let mut _res = true;
        let now = Instant::now();
        for (i, base) in seq.iter().filter_map(KT::from_nuc).enumerate() {
            if i < K - 1 {
                kmer = kmer.extend(base as KT);
            } else {
                kmer = kmer.append(base as KT);
                _res ^= set.contains(&kmer);
            }
        }
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/membership", elapsed / n);

        let now = Instant::now();
        for (i, base) in seq.iter().filter_map(KT::from_nuc).enumerate() {
            if i < K - 1 {
                kmer = kmer.extend(base as KT);
            } else {
                kmer = kmer.append(base as KT);
                set.remove(&kmer);
            }
        }
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/deletion", elapsed / n);
    });
}
