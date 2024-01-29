#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bincode::{DefaultOptions, Options};
use cbl::CBL;
use clap::Parser;
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
    first_index: String,
    /// Index file (CBL format)
    second_index: String,
    /// Output file (otherwise overwrite the first index file)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    let first_index_filename = args.first_index.as_str();
    let second_index_filename = args.second_index.as_str();
    let output_filename = if let Some(filename) = args.output {
        filename
    } else {
        first_index_filename.to_owned()
    };

    let first_index = File::open(first_index_filename)
        .unwrap_or_else(|_| panic!("Failed to open {first_index_filename}"));
    let reader = BufReader::new(first_index);
    eprintln!("Reading the first index stored in {first_index_filename}");
    let mut cbl: CBL<K, T, PREFIX_BITS> = DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize_from(reader)
        .unwrap();

    let second_index = File::open(second_index_filename)
        .unwrap_or_else(|_| panic!("Failed to open {second_index_filename}"));
    let reader = BufReader::new(second_index);
    eprintln!("Reading the second index stored in {second_index_filename}");
    let mut cbl2: CBL<K, T, PREFIX_BITS> = DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize_from(reader)
        .unwrap();

    cbl ^= &mut cbl2;

    let output = File::create(output_filename.as_str())
        .unwrap_or_else(|_| panic!("Failed to open {output_filename}"));
    let mut writer = BufWriter::new(output);
    eprintln!("Writing the updated first_index to {output_filename}");
    DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .serialize_into(&mut writer, &cbl)
        .unwrap();
}
