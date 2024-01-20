use cbl::CBL;
use clap::Parser;
use needletail::parse_fastx_file;

// Loads runtime-provided constants for which declarations
// will be generated at `$OUT_DIR/constants.rs`.
pub mod constants {
    include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

use constants::{K, PREFIX_BITS, T};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (FASTA/Q, possibly gzipped)
    input: String,
}

fn main() {
    let args = Args::parse();
    let input_filename = args.input.as_str();

    let mut cbl = CBL::<K, T, PREFIX_BITS>::new();
    let mut reader = parse_fastx_file(input_filename).expect("Failed to open {input_filename}");
    eprintln!("Building the index of {K}-mers contained in {input_filename}");
    while let Some(record) = reader.next() {
        let seqrec = record.expect("Invalid record");
        cbl.insert_seq(&seqrec.seq());
    }

    let mut total_load = 0.0;
    for (size, load) in cbl.buckets_load_repartition().iter() {
        total_load += load * 100.0;
        println!(
            "{:.3}% of items are in a bucket of size ≤ {size}",
            total_load
        );
    }
    let (max_prefix, max_size) = cbl.buckets_sizes().max_by_key(|&(_, size)| size).unwrap();
    println!("The biggest bucket (of size {max_size}) corresponds to prefix {max_prefix}");
}
