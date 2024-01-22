#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bincode::deserialize_from;
use cbl::CBL;
use clap::Parser;
use std::fs::File;
use std::io::BufReader;

// Loads runtime-provided constants for which declarations
// will be generated at `$OUT_DIR/constants.rs`.
pub mod constants {
    include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

use constants::{K, PREFIX_BITS, T};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Index file (CBL format)
    index: String,
}

fn main() {
    let args = Args::parse();
    let index_filename = args.index.as_str();

    let index = File::open(index_filename).expect("Failed to open {index_filename}");
    let reader = BufReader::new(index);
    eprintln!("Reading the index stored in {index_filename}");
    let cbl: CBL<K, T, PREFIX_BITS> = deserialize_from(reader).unwrap();

    let mut total_load = 0.0;
    for (size, load) in cbl.buckets_load_repartition().iter() {
        total_load += load * 100.0;
        eprintln!(
            "{:.3}% of items are in a bucket of size ≤ {size}",
            total_load
        );
    }
    let (max_prefix, max_size) = cbl.buckets_sizes().max_by_key(|&(_, size)| size).unwrap();
    eprintln!("The biggest bucket (of size {max_size}) corresponds to prefix {max_prefix}");
}
