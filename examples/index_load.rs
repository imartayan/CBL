#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bincode::{DefaultOptions, Options};
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

    let index =
        File::open(index_filename).unwrap_or_else(|_| panic!("Failed to open {index_filename}"));
    let reader = BufReader::new(index);
    eprintln!("Reading the index stored in {index_filename}");
    let cbl: CBL<K, T, PREFIX_BITS> = DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize_from(reader)
        .unwrap();

    eprintln!(
        "{:.1}% of the available prefixes are used",
        cbl.prefix_load() * 100.0
    );
    let buckets_size_count = cbl.buckets_size_count();
    let total_buckets: usize = buckets_size_count.iter().map(|(_, &c)| c).sum();
    let total_items: usize = buckets_size_count.iter().map(|(&s, &c)| s * c).sum();
    let mut bucket_count = 0;
    let mut item_count = 0;
    for (&size, &count) in buckets_size_count.iter() {
        bucket_count += count;
        item_count += size * count;
        if count > total_buckets / 1000
            || size * count > total_items / 1000
            || bucket_count == total_buckets
        {
            eprintln!(
                "{:.2}% of items are in a bucket of size ≤ {size} ({:.2}% of buckets)",
                (item_count * 100) as f64 / total_items as f64,
                (bucket_count * 100) as f64 / total_buckets as f64,
            );
        }
    }
    let (max_prefix, max_size) = cbl.buckets_sizes().max_by_key(|&(_, size)| size).unwrap();
    eprintln!("The biggest bucket (of size {max_size}) corresponds to prefix {max_prefix}");
}
