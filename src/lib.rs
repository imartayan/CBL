#![allow(incomplete_features)]
#![feature(slice_group_by)]
#![feature(generic_const_exprs)]

pub(crate) mod bitvector;
pub(crate) mod cbl;
pub(crate) mod compact_int;
pub(crate) mod ffi;
pub mod kmer;
pub(crate) mod necklace;
pub(crate) mod trie;
pub(crate) mod trie_vec;
pub(crate) mod wordset;

pub use cbl::CBL;
