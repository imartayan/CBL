#![allow(unused_imports)]

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
    generate!("TieredVec30")
    generate!("TieredVec32")

    safety!(unsafe)
}

pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiered_remove() {
        let tv = TieredVec28::new().within_unique_ptr();
        for i in 0..5 {
            tv.insert(i, i as u32);
        }
        for i in 0..5 {
            assert_eq!(tv.get(i), i as u32);
        }
        tv.remove(2);
        assert_eq!(tv.get(2), 3);
    }
}
