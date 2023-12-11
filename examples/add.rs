use cbl::bit_container::*;
use cbl::container::*;
use cbl::reads::*;
use cbl::CBL;
use clap::Parser;
use core::cmp::min;
use std::time::Instant;
// use cbl::kmer::*;

// Loads runtime-provided constants for which declarations
// will be generated at `$OUT_DIR/constants.rs`.
pub mod constants {
    include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

use constants::{K, KMER_BITS, KT, M, POS_BITS};
const N: usize = KMER_BITS; // canonize?
const W: usize = K - M + 1;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (.fasta, .fa)
    input: String,
}

type PrefixContainer = RankBitContainer;
// type SuffixContainer = PlainVec<u32>;
type SuffixContainer = SemiSortedVec<u32, 32>;

fn main() {
    let args = Args::parse();
    let input_filename = args.input.as_str();

    let mut cbl = CBL::<N, POS_BITS, W, K, KT, PrefixContainer, SuffixContainer>::new();
    let reads = Fasta::from_file(input_filename);

    // reads.process(|nucs| {
    //     let mut kmer = RawKmer::<K, KT>::new();
    //     for (i, base) in nucs.filter_map(KT::from_nuc).enumerate() {
    //         if i < K - 1 {
    //             kmer = kmer.extend(base as KT);
    //         } else {
    //             kmer = kmer.append(base as KT);
    //             cbl.insert(kmer);
    //             // assert!(cbl.contains(kmer));
    //             // debug_assert!(cbl.contains(kmer));
    //         }
    //     }
    // });

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
        // for (size, proportion) in cbl.suffix_load_repartition() {
        //     let percent = (proportion * 100.0).round();
        //     if percent > 0.0 {
        //         println!("size {size}: {percent} %");
        //     }
        // }

        // let mut _res = true;
        // let now = Instant::now();
        // for start in (0..(seq.len() - K + 1)).step_by(chunk_size) {
        //     _res ^= cbl.contains_seq(&seq[start..(min(start + chunk_size + K - 1, seq.len()))]);
        // }
        // let elapsed = now.elapsed().as_nanos();
        // println!("{} ns/membership", elapsed / n);

        let now = Instant::now();
        for start in (0..(seq.len() - K + 1)).step_by(chunk_size) {
            cbl.remove_seq(&seq[start..(min(start + chunk_size + K - 1, seq.len()))]);
        }
        let elapsed = now.elapsed().as_nanos();
        println!("{} ns/deletion", elapsed / n);
    });
}
