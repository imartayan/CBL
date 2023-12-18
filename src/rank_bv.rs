pub use cxx::UniquePtr;
pub use rbv::*;

#[cxx::bridge]
mod rbv {
    unsafe extern "C++" {
        include!("CBL/cxx/rank_bv.h");

        type RankBV;
        fn new_rank_bv(size: usize) -> UniquePtr<RankBV>;
        fn size(&self) -> usize;
        fn get(&self, index: usize) -> bool;
        fn set(&self, index: usize) -> bool;
        fn clear(&self, index: usize) -> bool;
        fn toggle(&self, index: usize) -> bool;
        fn rank(&self, index: usize) -> u64;
        fn count_ones(&self) -> usize;
    }
}
