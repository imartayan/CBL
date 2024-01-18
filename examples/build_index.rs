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

use constants::{K, M, NT, PREFIX_BITS};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (.fasta, .fa)
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

    let mut cbl = CBL::<K, NT, PREFIX_BITS, M>::new();
    let mut reader = parse_fastx_file(&input_filename).expect("Failed to open input file");
    while let Some(record) = reader.next() {
        let seqrec = record.expect("Invalid record");
        cbl.insert_seq(&seqrec.seq());
    }

    let output = File::create(output_filename).expect("Failed to open output file");
    let mut writer = BufWriter::new(output);
    serialize_into(&mut writer, &cbl).unwrap();
}
