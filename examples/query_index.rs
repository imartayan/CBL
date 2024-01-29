#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bincode::{DefaultOptions, Options};
use cbl::CBL;
use clap::Parser;
use needletail::parse_fastx_file;
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
    /// Input file to query (FASTA/Q, possibly gzipped)
    input: String,
}

fn main() {
    let args = Args::parse();
    let index_filename = args.index.as_str();
    let input_filename = args.input.as_str();

    let index =
        File::open(index_filename).unwrap_or_else(|_| panic!("Failed to open {index_filename}"));
    let reader = BufReader::new(index);
    eprintln!("Reading the index stored in {index_filename}");
    let mut cbl: CBL<K, T, PREFIX_BITS> = DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize_from(reader)
        .unwrap();

    let mut reader = parse_fastx_file(input_filename)
        .unwrap_or_else(|_| panic!("Failed to open {input_filename}"));
    eprintln!("Querying the {K}-mers contained in {input_filename}");
    let mut total = 0usize;
    let mut positive = 0usize;
    while let Some(record) = reader.next() {
        let seqrec = record.expect("Invalid record");
        let contained = cbl.contains_seq(&seqrec.seq());
        total += contained.len();
        for p in contained {
            if p {
                positive += 1;
            }
        }
    }
    eprintln!("# queries: {total}");
    eprintln!(
        "# positive queries: {positive} ({:.2}%)",
        (positive * 100) as f64 / total as f64
    );
}
