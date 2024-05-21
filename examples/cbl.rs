#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use bincode::{DefaultOptions, Options};
use cbl::{kmer::Kmer, CBL};
use clap::{Args, Parser, Subcommand};
use needletail::{parse_fastx_file, FastxReader};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Write};
use std::path::Path;

// Loads runtime-provided constants for which declarations
// will be generated at `$OUT_DIR/constants.rs`.
pub mod constants {
    include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

use constants::{K, PREFIX_BITS, T};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build an index containing the k-mers of a FASTA/Q file
    Build(BuildArgs),
    /// Count the k-mers contained in an index
    Count(IndexArgs),
    /// List the k-mers contained in an index
    List(ListArgs),
    /// Query an index for every k-mer contained in a FASTA/Q file
    Query(QueryArgs),
    /// Add the k-mers of a FASTA/Q file to an index
    Insert(UpdateArgs),
    /// Remove the k-mers of a FASTA/Q file from an index
    Remove(UpdateArgs),
    /// Compute the union of two indexes
    Merge(SetOpsArgs),
    /// Compute the intersection of two indexes
    Inter(SetOpsArgs),
    /// Compute the difference of two indexes
    Diff(SetOpsArgs),
    /// Compute the symmetric difference of two indexes
    SymDiff(SetOpsArgs),
    /// Show the repartition of the k-mers in the data structure
    Repartition(IndexArgs),
}

#[derive(Args, Debug)]
struct BuildArgs {
    /// Input file (FASTA/Q, possibly gzipped)
    input: String,
    /// Output file (no serialization by default)
    #[arg(short, long)]
    output: Option<String>,
    /// Use canonical k-mers
    #[arg(short, long)]
    canonical: bool,
}

#[derive(Args, Debug)]
struct IndexArgs {
    /// Index file (CBL format)
    index: String,
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Index file (CBL format)
    index: String,
    /// Output file (write to stdout by default)
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Args, Debug)]
struct QueryArgs {
    /// Index file (CBL format)
    index: String,
    /// Input file to query (FASTA/Q, possibly gzipped)
    input: String,
}

#[derive(Args, Debug)]
struct UpdateArgs {
    /// Index file (CBL format)
    index: String,
    /// Input file to query (FASTA/Q, possibly gzipped)
    input: String,
    /// Output file (no serialization by default)
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Args, Debug)]
struct SetOpsArgs {
    /// Index file (CBL format)
    first_index: String,
    /// Index file (CBL format)
    second_index: String,
    /// Output file (no serialization by default)
    #[arg(short, long)]
    output: Option<String>,
}

fn read_fasta<P: AsRef<Path> + Copy>(path: P) -> Box<dyn FastxReader> {
    parse_fastx_file(path)
        .unwrap_or_else(|_| panic!("Failed to open {}", path.as_ref().to_str().unwrap()))
}

fn read_index<D: DeserializeOwned, P: AsRef<Path> + Copy>(path: P) -> D {
    let index = File::open(path)
        .unwrap_or_else(|_| panic!("Failed to open {}", path.as_ref().to_str().unwrap()));
    let reader = BufReader::new(index);
    eprintln!(
        "Reading the index stored in {}",
        path.as_ref().to_str().unwrap()
    );
    DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .deserialize_from(reader)
        .unwrap()
}

fn write_index<S: Serialize, P: AsRef<Path> + Copy>(index: &S, path: P) {
    let output = File::create(path)
        .unwrap_or_else(|_| panic!("Failed to open {}", path.as_ref().to_str().unwrap()));
    let mut writer = BufWriter::new(output);
    eprintln!("Writing the index to {}", path.as_ref().to_str().unwrap());
    DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
        .serialize_into(&mut writer, &index)
        .unwrap();
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Command::Build(args) => {
            let input_filename = args.input.as_str();
            let mut cbl = if args.canonical {
                CBL::<K, T, PREFIX_BITS>::new_canonical()
            } else {
                CBL::<K, T, PREFIX_BITS>::new()
            };
            let mut reader = read_fasta(input_filename);
            if cbl.is_canonical() {
                eprintln!("Building the index of canonical {K}-mers contained in {input_filename}");
            } else {
                eprintln!("Building the index of {K}-mers contained in {input_filename}");
            }
            while let Some(record) = reader.next() {
                let seqrec = record.unwrap_or_else(|_| panic!("Invalid record"));
                cbl.insert_seq(&seqrec.seq());
            }
            if let Some(output_filename) = args.output {
                write_index(&cbl, output_filename.as_str());
            }
        }
        Command::Count(args) => {
            let index_filename = args.index.as_str();
            let cbl: CBL<K, T, PREFIX_BITS> = read_index(index_filename);
            if cbl.is_canonical() {
                eprintln!("It contains {} canonical {K}-mers", cbl.count());
            } else {
                eprintln!("It contains {} {K}-mers", cbl.count());
            }
        }
        Command::List(args) => {
            let index_filename = args.index.as_str();
            let cbl: CBL<K, T, PREFIX_BITS> = read_index(index_filename);
            if cbl.is_canonical() {
                eprintln!("Listing canonical {K}-mers contained in {index_filename}");
            } else {
                eprintln!("Listing {K}-mers contained in {index_filename}");
            }
            if let Some(output_filename) = args.output {
                let output_filename = output_filename.as_str();
                let file = File::create(output_filename)
                    .unwrap_or_else(|_| panic!("Failed to open {}", output_filename));
                let mut writer = BufWriter::new(file);
                for kmer in cbl.iter() {
                    writer.write_all(&kmer.to_nucs()).unwrap();
                    writer.write_all(b"\n").unwrap();
                }
            } else {
                let mut writer = stdout().lock();
                for kmer in cbl.iter() {
                    writer.write_all(&kmer.to_nucs()).unwrap();
                    writer.write_all(b"\n").unwrap();
                }
            }
        }
        Command::Query(args) => {
            let index_filename = args.index.as_str();
            let input_filename = args.input.as_str();
            let mut cbl: CBL<K, T, PREFIX_BITS> = read_index(index_filename);
            let mut reader = read_fasta(input_filename);
            if cbl.is_canonical() {
                eprintln!("Querying the canonical {K}-mers contained in {input_filename}");
            } else {
                eprintln!("Querying the {K}-mers contained in {input_filename}");
            }
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
        Command::Insert(args) => {
            let index_filename = args.index.as_str();
            let input_filename = args.input.as_str();
            let mut cbl: CBL<K, T, PREFIX_BITS> = read_index(index_filename);
            let mut reader = read_fasta(input_filename);
            if cbl.is_canonical() {
                eprintln!(
                    "Adding the canonical {K}-mers contained in {input_filename} to the index"
                );
            } else {
                eprintln!("Adding the {K}-mers contained in {input_filename} to the index");
            }
            while let Some(record) = reader.next() {
                let seqrec = record.expect("Invalid record");
                cbl.insert_seq(&seqrec.seq());
            }
            if let Some(output_filename) = args.output {
                write_index(&cbl, output_filename.as_str());
            }
        }
        Command::Remove(args) => {
            let index_filename = args.index.as_str();
            let input_filename = args.input.as_str();
            let mut cbl: CBL<K, T, PREFIX_BITS> = read_index(index_filename);
            let mut reader = read_fasta(input_filename);
            if cbl.is_canonical() {
                eprintln!(
                    "Removing the canonical {K}-mers contained in {input_filename} from the index"
                );
            } else {
                eprintln!("Removing the {K}-mers contained in {input_filename} from the index");
            }
            while let Some(record) = reader.next() {
                let seqrec = record.expect("Invalid record");
                cbl.remove_seq(&seqrec.seq());
            }
            if let Some(output_filename) = args.output {
                write_index(&cbl, output_filename.as_str());
            }
        }
        Command::Merge(args) => {
            let first_index_filename = args.first_index.as_str();
            let second_index_filename = args.second_index.as_str();
            let mut cbl: CBL<K, T, PREFIX_BITS> = read_index(first_index_filename);
            let mut cbl2: CBL<K, T, PREFIX_BITS> = read_index(second_index_filename);
            cbl |= &mut cbl2;
            if let Some(output_filename) = args.output {
                write_index(&cbl, output_filename.as_str());
            }
        }
        Command::Inter(args) => {
            let first_index_filename = args.first_index.as_str();
            let second_index_filename = args.second_index.as_str();
            let mut cbl: CBL<K, T, PREFIX_BITS> = read_index(first_index_filename);
            let mut cbl2: CBL<K, T, PREFIX_BITS> = read_index(second_index_filename);
            cbl &= &mut cbl2;
            if let Some(output_filename) = args.output {
                write_index(&cbl, output_filename.as_str());
            }
        }
        Command::Diff(args) => {
            let first_index_filename = args.first_index.as_str();
            let second_index_filename = args.second_index.as_str();
            let mut cbl: CBL<K, T, PREFIX_BITS> = read_index(first_index_filename);
            let mut cbl2: CBL<K, T, PREFIX_BITS> = read_index(second_index_filename);
            cbl -= &mut cbl2;
            if let Some(output_filename) = args.output {
                write_index(&cbl, output_filename.as_str());
            }
        }
        Command::SymDiff(args) => {
            let first_index_filename = args.first_index.as_str();
            let second_index_filename = args.second_index.as_str();
            let mut cbl: CBL<K, T, PREFIX_BITS> = read_index(first_index_filename);
            let mut cbl2: CBL<K, T, PREFIX_BITS> = read_index(second_index_filename);
            cbl ^= &mut cbl2;
            if let Some(output_filename) = args.output {
                write_index(&cbl, output_filename.as_str());
            }
        }
        Command::Repartition(args) => {
            let index_filename = args.index.as_str();
            let cbl: CBL<K, T, PREFIX_BITS> = read_index(index_filename);
            eprintln!(
                "{:.1}% of the available prefixes are used",
                cbl.prefix_load() * 100.0
            );
            let buckets_size_count = cbl.buckets_size_count();
            let total_buckets: usize = buckets_size_count.iter().map(|(_, &c)| c).sum();
            let total_items: usize = buckets_size_count.iter().map(|(&s, &c)| s * c).sum();
            eprintln!(
                "The average bucket size is {:.1} items",
                total_items as f64 / total_buckets as f64
            );
            let mut bucket_count = 0;
            let mut item_count = 0;
            for (&size, &count) in buckets_size_count.iter() {
                bucket_count += count;
                item_count += size * count;
                if count > total_buckets / 100 / 2
                    || size * count > total_items / 100 / 2
                    || bucket_count == total_buckets
                {
                    eprintln!(
                        "{:.1}% of items are in a bucket of size ≤ {size} ({:.1}% of buckets)",
                        (item_count * 100) as f64 / total_items as f64,
                        (bucket_count * 100) as f64 / total_buckets as f64,
                    );
                }
            }
            let (max_prefix, max_size) = cbl.buckets_sizes().max_by_key(|&(_, size)| size).unwrap();
            eprintln!("The biggest bucket (of size {max_size}) corresponds to prefix {max_prefix}");
        }
    }
}
