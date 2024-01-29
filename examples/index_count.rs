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
    eprintln!("It contains {} {K}-mers", cbl.count());
}
