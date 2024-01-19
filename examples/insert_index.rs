#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bincode::{deserialize_from, serialize_into};
use cbl::CBL;
use clap::Parser;
use needletail::parse_fastx_file;
use std::fs::File;
use std::io::{BufReader, BufWriter};

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
    /// Input file to insert (FASTA/Q, possibly gzipped)
    input: String,
    /// Output file (otherwise overwrite the index file)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    let index_filename = args.index.as_str();
    let input_filename = args.input.as_str();
    let output_filename = if let Some(filename) = args.output {
        filename
    } else {
        index_filename.to_owned()
    };

    let index = File::open(index_filename).expect("Failed to open index file");
    let reader = BufReader::new(index);
    let mut cbl: CBL<K, T, PREFIX_BITS> = deserialize_from(reader).unwrap();

    let mut reader = parse_fastx_file(input_filename).expect("Failed to open input file");
    while let Some(record) = reader.next() {
        let seqrec = record.expect("Invalid record");
        cbl.insert_seq(&seqrec.seq());
    }

    let output = File::create(output_filename).expect("Failed to open output file");
    let mut writer = BufWriter::new(output);
    serialize_into(&mut writer, &cbl).unwrap();
}
