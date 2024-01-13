#![allow(incomplete_features)]
#![feature(slice_group_by)]
#![feature(generic_const_exprs)]

pub(crate) mod bit_container;
pub(crate) mod compact_int;
pub(crate) mod container;
pub(crate) mod ffi;
pub(crate) mod necklace;
pub(crate) mod wordset;

pub mod cbl;
pub mod kmer;
pub mod reads;

pub use cbl::*;
