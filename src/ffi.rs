#![allow(dead_code)]

use autocxx::prelude::*;
pub use autocxx::WithinUniquePtr;
pub use cxx::UniquePtr;

include_cpp! {
    #include "rank_bv.h"
    #include "tiered_vec.h"

    generate!("RankBV")
    generate!("TieredVec16")
    generate!("TieredVec20")
    generate!("TieredVec24")
    generate!("TieredVec28")
    generate!("TieredVec32")

    safety!(unsafe)
}

pub use ffi::*;
