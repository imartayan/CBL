# CBL

A Rust library providing fully dynamic sets of *k*-mers with high [locality](https://en.wikipedia.org/wiki/Locality_of_reference).

The `CBL` data structure supports the following operations:
- inserting a single *k*-mer (with `insert`), or every *k*-mer from a sequence (with `insert_seq`)
- deleting a single *k*-mer (with `remove`), or every *k*-mer from a sequence (with `remove_seq`)
- membership of a single *k*-mer (with `contains`), or every *k*-mer from a sequence (with `contains_seq`)
- iterating over the *k*-mers stored in the set (with `iter`)
- union / intersection / difference of two sets (with `|` / `&` / `-`)
- (de)serialization with [serde](https://serde.rs/)

You can see some examples of how to use it in the `examples` folder.

## Requirements

### Rust nightly

If you have not installed Rust yet, please visit [rustup.rs](https://rustup.rs/) to install it.
This library uses some nightly features of the Rust compiler, you can install the latest nightly version with
```sh
rustup install nightly
```

If you don't want to use the `+nightly` flag every time you run `cargo`, you can set it as default with
```sh
rustup default nightly
```

### Additional headers for Linux

This library uses C++ bindings for the [sux](https://github.com/vigna/sux) library and [tiered vectors](https://github.com/mettienne/tiered-vector).
Depending on your configuration, some headers used for the bindings might be missing, in that case please install the following packages
```sh
sudo apt install -y libstdc++-12-dev libclang-dev
```

## Using the library with cargo

You can add `CBL` in an existing Rust project with
```sh
cargo +nightly add --git https://github.com/imartayan/CBL.git
```
or by adding the following dependency in your `Cargo.toml`
```toml
cbl = { git = https://github.com/imartayan/CBL.git }
```

### Choosing the right parameters

The `CBL` struct takes two main parameters as constants:
- an integer `K` specifying the size of the *k*-mers
- an integer type `T` (e.g. `u32`, `u64`, `u128`) that must be large enough to store both a *k*-mer *and* its number of bits together

Therefore `T` should be large enough to store $2k + \lg(2k)$ bits.
In particular, since primitive integers cannot store more than 128 bits, this means that `K` must be ≤ 59.

Additionally, you can specify a third (optional) parameter `PREFIX_BITS` which determines the size of the underlying bitvector.
Changing this parameter affects the space usage and the query time of the data structure, see the paper for more details.

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
