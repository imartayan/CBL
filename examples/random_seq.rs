use cbl::kmer::Base;
use clap::Parser;
use rand::Rng;
use std::fs::File;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Length of the sequence
    #[arg(short, long, default_value_t = 1_000_000)]
    len: usize,
    /// Output file (.fasta, .fa)
    #[arg(short, long, default_value_t = String::from("random_seq.fa"))]
    output: String,
}

fn main() {
    let args = Args::parse();
    let mut output = File::create(args.output).expect("Failed to open output file");
    let mut bases = Vec::with_capacity(args.len + 2);
    bases.extend_from_slice(b">\n");
    let mut rng = rand::thread_rng();
    for _ in 0..args.len {
        bases.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
    }
    output.write_all(&bases).expect("Failed to write bases");
}
