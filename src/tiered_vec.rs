#![allow(dead_code)]

pub use cxx::UniquePtr;
use std::fmt::Debug;

#[cxx::bridge]
mod tv32 {
    unsafe extern "C++" {
        include!("CBL/cxx/tiered_vec.h");

        type TieredVec32;
        fn new_tiered_vec_32() -> UniquePtr<TieredVec32>;
        fn len(&self) -> usize;
        fn is_empty(&self) -> bool;
        fn capacity(&self) -> usize;
        fn get(&self, idx: usize) -> u32;
        fn update(&self, idx: usize, elem: u32) -> u32;
        fn insert(&self, idx: usize, elem: u32);
        fn remove(&self, idx: usize);
        fn insert_sorted(&self, elem: u32);
        fn contains_sorted(&self, elem: u32) -> bool;
        fn index_sorted(&self, elem: u32) -> usize;
    }
}
pub use tv32::*;

#[cxx::bridge]
mod tv28 {
    unsafe extern "C++" {
        include!("CBL/cxx/tiered_vec.h");

        type TieredVec28;
        fn new_tiered_vec_28() -> UniquePtr<TieredVec28>;
        fn len(&self) -> usize;
        fn is_empty(&self) -> bool;
        fn capacity(&self) -> usize;
        fn get(&self, idx: usize) -> u32;
        fn update(&self, idx: usize, elem: u32) -> u32;
        fn insert(&self, idx: usize, elem: u32);
        fn remove(&self, idx: usize);
        fn insert_sorted(&self, elem: u32);
        fn contains_sorted(&self, elem: u32) -> bool;
        fn index_sorted(&self, elem: u32) -> usize;
    }
}
pub use tv28::*;

impl Debug for TieredVec28 {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[cxx::bridge]
mod tv24 {
    unsafe extern "C++" {
        include!("CBL/cxx/tiered_vec.h");

        type TieredVec24;
        fn new_tiered_vec_24() -> UniquePtr<TieredVec24>;
        fn len(&self) -> usize;
        fn is_empty(&self) -> bool;
        fn capacity(&self) -> usize;
        fn get(&self, idx: usize) -> u32;
        fn update(&self, idx: usize, elem: u32) -> u32;
        fn insert(&self, idx: usize, elem: u32);
        fn remove(&self, idx: usize);
        fn insert_sorted(&self, elem: u32);
        fn contains_sorted(&self, elem: u32) -> bool;
        fn index_sorted(&self, elem: u32) -> usize;
    }
}
pub use tv24::*;

#[cxx::bridge]
mod tv20 {
    unsafe extern "C++" {
        include!("CBL/cxx/tiered_vec.h");

        type TieredVec20;
        fn new_tiered_vec_20() -> UniquePtr<TieredVec20>;
        fn len(&self) -> usize;
        fn is_empty(&self) -> bool;
        fn capacity(&self) -> usize;
        fn get(&self, idx: usize) -> u32;
        fn update(&self, idx: usize, elem: u32) -> u32;
        fn insert(&self, idx: usize, elem: u32);
        fn remove(&self, idx: usize);
        fn insert_sorted(&self, elem: u32);
        fn contains_sorted(&self, elem: u32) -> bool;
        fn index_sorted(&self, elem: u32) -> usize;
    }
}
pub use tv20::*;

#[cxx::bridge]
mod tv16 {
    unsafe extern "C++" {
        include!("CBL/cxx/tiered_vec.h");

        type TieredVec16;
        fn new_tiered_vec_16() -> UniquePtr<TieredVec16>;
        fn len(&self) -> usize;
        fn is_empty(&self) -> bool;
        fn capacity(&self) -> usize;
        fn get(&self, idx: usize) -> u16;
        fn update(&self, idx: usize, elem: u16) -> u16;
        fn insert(&self, idx: usize, elem: u16);
        fn remove(&self, idx: usize);
        fn insert_sorted(&self, elem: u16);
        fn contains_sorted(&self, elem: u16) -> bool;
        fn index_sorted(&self, elem: u16) -> usize;
    }
}
pub use tv16::*;

// use paste::paste;

// macro_rules! ffi_tv {
//     ($size:literal, $T:ty) => {
//         paste! {
//             // #[cxx::bridge]
//             mod [<tv $size>] {
//                 unsafe extern "C++" {
//                     include!("CBL/cxx/tiered_vec.h");

//                     type [<TieredVec $size>];
//                     fn [<new_tiered_vec_ $size>]() -> UniquePtr<[<TieredVec $size>]>;
//                     fn len(&self) -> usize;
//                     fn is_empty(&self) -> bool;
//                     fn capacity(&self) -> usize;
//                     fn get(&self, idx: usize) -> $T;
//                     fn update(&self, idx: usize, elem: $T) -> $T;
//                     fn insert(&self, idx: usize, elem: $T);
//                     fn remove(&self, idx: usize);
//                     fn insert_sorted(&self, elem: $T);
//                     fn contains_sorted(&self, elem: $T) -> bool;
//                 }
//             }

//             pub use [<tv $size>]::*;
//         }
//     };
// }

// ffi_tv!(28, u32);
