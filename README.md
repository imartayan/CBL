# Conway-Bromage-Lyndon

A Rust library providing fully dynamic sets of *k*-mers with high [locality](https://en.wikipedia.org/wiki/Locality_of_reference).

The data structure is described in [Conway-Bromage-Lyndon (CBL): an exact, dynamic representation of k-mer sets](https://doi.org/10.1093/bioinformatics/btae217), please [cite it](#citation) if you use this library.

It supports the following operations:
- inserting a single *k*-mer (with `insert`), or every *k*-mer from a sequence (with `insert_seq`)
- deleting a single *k*-mer (with `remove`), or every *k*-mer from a sequence (with `remove_seq`)
- membership of a single *k*-mer (with `contains`), or every *k*-mer from a sequence (with `contains_seq`)
- iterating over the *k*-mers stored in the set (with `iter`)
- union / intersection / difference of two sets (with `|` / `&` / `-`)
- (de)serialization with [serde](https://serde.rs/)

## Requirements

### Rust nightly 1.77+

If you have not installed Rust yet, please visit [rustup.rs](https://rustup.rs/) to install it.
This library uses some nightly features of the Rust compiler (version 1.77+), you can install the latest nightly version with
```sh
rustup install nightly
```

If you don't want to use the `+nightly` flag every time you run `cargo`, you can set it as default with
```sh
rustup default nightly
```

### Additional headers for Linux

This library uses C++ bindings for the [sux](https://github.com/vigna/sux) library and [tiered vectors](https://github.com/mettienne/tiered-vector).
Depending on your configuration, some headers used for the bindings might be missing, in that case please install the following packages:

#### Ubuntu

```sh
sudo apt install -y libstdc++-12-dev libclang-dev
```

#### Fedora

```sh
sudo dnf install -y clang15-devel
```

## Using the library

You can add `CBL` in an existing Rust project with
```sh
cargo +nightly add --git https://github.com/imartayan/CBL.git
```
or by adding the following dependency in your `Cargo.toml`
```toml
cbl = { git = "https://github.com/imartayan/CBL.git" }
```
If the build fails, try to install [additional headers](#additional-headers-for-linux).

### Choosing the right parameters

The `CBL` struct takes two main parameters as constants:
- an integer `K` specifying the size of the *k*-mers
- an integer type `T` (e.g. `u32`, `u64`, `u128`) that must be large enough to store both a *k*-mer *and* its number of bits together

Therefore `T` should be large enough to store $2k + \lg(2k)$ bits.
In particular, since primitive integers cannot store more than 128 bits, this means that `K` must be ≤ 59.

Additionally, you can specify a third (optional) parameter `PREFIX_BITS` which determines the size of the underlying bitvector.
Changing this parameter affects the space usage and the query time of the data structure, see the paper for more details.

### Example usage

```rs
use cbl::CBL;
use needletail::parse_fastx_file;
use std::env::args;

// define the parameters K and T
const K: usize = 25;
type T = u64; // T must be large enough to store $2k + \lg(2k)$ bits

fn main() {
    let args: Vec<String> = args().collect();
    let input_filename = args.get(1).expect("No argument given");

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
```

## Building from source

You can clone the repository and its submodules with
```sh
git clone --recursive https://github.com/imartayan/CBL.git
```

If you did not use the `--recursive` flag, make sure to load the submodules with
```sh
git submodule update --init --recursive
```

### Running the binaries

You can compile the binaries with
```sh
cargo +nightly build --release --examples
```
If the build fails, try to install [additional headers](#additional-headers-for-linux).

By default, the binaries are compiled with a fixed `K` equal to 25, you can compile them with a different `K` as follows
```sh
K=59 cargo +nightly build --release --examples
```
Note that `K` values ≥ 60 are not supported by this library.

Similarly, `PREFIX_BITS` is equal to 24 by default and you can change it with
```sh
K=59 PREFIX_BITS=28 cargo +nightly build --release --examples
```
Note that `PREFIX_BITS` values ≥ 29 are not supported by this library.

Once compiled, the main binary will be located at `target/release/examples/cbl`.
It supports the following commands:
```md
Usage: cbl <COMMAND>

Commands:
  build        Build an index containing the k-mers of a FASTA/Q file
  count        Count the k-mers contained in an index
  list         List the k-mers contained in an index
  query        Query an index for every k-mer contained in a FASTA/Q file
  insert       Add the k-mers of a FASTA/Q file to an index
  remove       Remove the k-mers of a FASTA/Q file from an index
  merge        Compute the union of two indexes
  inter        Compute the intersection of two indexes
  diff         Compute the difference of two indexes
  sym-diff     Compute the symmetric difference of two indexes
  repartition  Show the repartition of the k-mers in the data structure
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Running the tests

You can run all the tests with
```sh
cargo +nightly test --lib
```

### Building the documentation

You can build the documentation of the library and open it in your browser with
```sh
cargo +nightly doc --lib --no-deps --open
```

## Citation

> Conway-Bromage-Lyndon (CBL): an exact, dynamic representation of k-mer sets. Martayan, I., Cazaux, B., Limasset, A., and Marchet, C. https://doi.org/10.1093/bioinformatics/btae217

```bibtex
@article{cbl,
  title   = {{Conway–Bromage–Lyndon (CBL): an exact, dynamic representation of k-mer sets}},
  author  = {Martayan, Igor and Cazaux, Bastien and Limasset, Antoine and Marchet, Camille},
  journal = {Bioinformatics},
  volume  = {40},
  number  = {Supplement_1},
  pages   = {i48-i57},
  year    = {2024},
  month   = {06},
  issn    = {1367-4811},
  doi     = {10.1093/bioinformatics/btae217},
  url     = {https://doi.org/10.1093/bioinformatics/btae217},
  eprint  = {https://academic.oup.com/bioinformatics/article-pdf/40/Supplement\_1/i48/58354678/btae217.pdf}
}
```
