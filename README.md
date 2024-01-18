# CBL

A Rust library providing fully dynamic sets of *k*-mers with high [locality](https://en.wikipedia.org/wiki/Locality_of_reference).

The `CBL` data structure supports the following operations:
- inserting a single *k*-mer with `insert`, or every *k*-mer from a sequence with `insert_seq`
- deleting a single *k*-mer with `remove`, or every *k*-mer from a sequence with `remove_seq`
- membership of a single *k*-mer with `contains`, or every *k*-mer from a sequence with `contains_seq`
- iterating over the *k*-mers stored in the set with `iter`
- union of two sets with the `|` operator
- intersection of two sets with the `&` operator
- difference of two sets with the `-` operator
- symmetric difference of two sets with the `^` operator
- (de)serialization with `serde`

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

### Extra headers for Linux

This library uses C++ bindings for the [sux](https://github.com/vigna/sux) library and [tiered vectors](https://github.com/mettienne/tiered-vector).
Depending on your configuration, some headers used for the bindings might be missing, in that case please install the following packages
```sh
apt install -y libstdc++-12-dev libclang-dev
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

## Building from source

You can clone the repository and its submodules with
```sh
git clone --recursive https://github.com/imartayan/CBL.git
```

If you did not use the `--recursive` flag, make sure to load the submodules with
```sh
git submodule update --init --recursive
```

### Running the examples

You can compile the examples with
```sh
cargo +nightly build --release --examples
```

By default, the examples are compiled with a fixed `K` equal to 25, you can compile them with a different `K` as follows
```sh
K=59 cargo +nightly build --release --examples
```

Once compiled, the binaries will be located in `target/release/examples`.
- `build_index <input>` creates an index containing the *k*-mers of a FASTA/Q file, and serialize it on disk.
- `insert_index <index> <input>` adds the *k*-mers of a FASTA/Q file to a given index.
- `remove_index <index> <input>` removes the *k*-mers of a FASTA/Q file to a given index.
- `validate_index <index> <input>` checks that all the *k*-mers of a FASTA/Q file are contained in a given index.

### Running the tests

You can run all the tests with
```sh
cargo +nightly test --lib
```
