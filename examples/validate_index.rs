#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bincode::deserialize_from;
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

use constants::{K, M, NT, PREFIX_BITS};

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

    let index = File::open(&index_filename).expect("Failed to open index file");
    let reader = BufReader::new(index);
    let mut cbl: CBL<K, NT, PREFIX_BITS, M> = deserialize_from(reader).unwrap();

    let mut reader = parse_fastx_file(&input_filename).expect("Failed to open input file");
    while let Some(record) = reader.next() {
        let seqrec = record.expect("Invalid record");
        assert!(cbl.contains_all(&seqrec.seq()));
    }
}
