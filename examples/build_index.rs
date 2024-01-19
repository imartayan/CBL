use bincode::serialize_into;
use cbl::CBL;
use clap::Parser;
use needletail::parse_fastx_file;
use std::fs::File;
use std::io::BufWriter;

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
    /// Output file (defaults to <input>.cbl)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    let input_filename = args.input.as_str();
    let output_filename = if let Some(filename) = args.output {
        filename
    } else {
        input_filename.to_owned() + ".cbl"
    };

    let mut cbl = CBL::<K, T, PREFIX_BITS>::new();
    let mut reader = parse_fastx_file(input_filename).expect("Failed to open {input_filename}");
    eprintln!("Building the index of {K}-mers contained in {input_filename}");
    while let Some(record) = reader.next() {
        let seqrec = record.expect("Invalid record");
        cbl.insert_seq(&seqrec.seq());
    }

    let output = File::create(output_filename.as_str()).expect("Failed to open {output_filename}");
    let mut writer = BufWriter::new(output);
    eprintln!("Writing the index to {output_filename}");
    serialize_into(&mut writer, &cbl).unwrap();
}
