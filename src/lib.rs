#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub(crate) mod bitvector;
pub mod cbl;
pub(crate) mod ffi;
pub mod kmer;
pub mod necklace;
pub(crate) mod sliced_int;
pub(crate) mod trie;
pub(crate) mod trievec;
pub(crate) mod wordset;

pub use cbl::CBL;
