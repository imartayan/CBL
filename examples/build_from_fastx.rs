use cbl::CBL;
use needletail::parse_fastx_file;
use std::env::args;

// define the parameters K and T
const K: usize = 25;
type T = u64; // T must be large enough to store $2k + \lg(2k)$ bits

fn main() {
    let args: Vec<String> = args().collect();
    let input_filename = &args.get(1).expect("No argument given");

    // create a CBL index with parameters K and T
    let mut cbl = CBL::<K, T>::new();

    let mut reader = parse_fastx_file(input_filename).unwrap();
    // for each sequence of the FASTA/Q file
    while let Some(record) = reader.next() {
        let seqrec = record.expect("Invalid record");

        // insert each k-mer of the sequence in the index
        cbl.insert_seq(&seqrec.seq());
    }
}
