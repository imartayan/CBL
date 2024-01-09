#![allow(incomplete_features)]
#![feature(slice_group_by)]
#![feature(generic_const_exprs)]

pub mod bit_container;
pub mod cbl;
pub mod compact_int;
pub mod container;
pub mod ffi;
pub mod kmer;
pub mod necklace;
pub mod reads;
pub mod wordset;

pub use cbl::*;
