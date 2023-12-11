use crate::rank_bv::{new_rank_bv, RankBV, UniquePtr};
use roaring::RoaringBitmap;

pub trait BitContainer {
    fn new_with_len(len: usize) -> Self;
    fn contains(&self, index: usize) -> bool;
    fn insert(&mut self, index: usize) -> bool;
    fn remove(&mut self, index: usize) -> bool;
    fn rank(&self, index: usize) -> usize;
    fn count(&self) -> usize;
}

pub struct RoaringBitContainer {
    roaring: RoaringBitmap,
}

impl BitContainer for RoaringBitContainer {
    #[inline]
    fn new_with_len(len: usize) -> Self {
        assert!(len <= 32, "Roaring supports up to 32 bits");
        Self {
            roaring: RoaringBitmap::new(),
        }
    }

    #[inline]
    fn contains(&self, index: usize) -> bool {
        self.roaring.contains(index as u32)
    }

    #[inline]
    fn insert(&mut self, index: usize) -> bool {
        self.roaring.insert(index as u32)
    }

    #[inline]
    fn remove(&mut self, index: usize) -> bool {
        self.roaring.remove(index as u32)
    }

    #[inline]
    fn rank(&self, index: usize) -> usize {
        self.roaring.rank(index as u32) as usize - 1
    }

    #[inline]
    fn count(&self) -> usize {
        self.roaring.len() as usize
    }
}

pub struct RankBitContainer {
    bv: UniquePtr<RankBV>,
}

impl BitContainer for RankBitContainer {
    #[inline]
    fn new_with_len(len: usize) -> Self {
        Self {
            bv: new_rank_bv(1 << len),
        }
    }

    #[inline]
    fn contains(&self, index: usize) -> bool {
        self.bv.get(index)
    }

    #[inline]
    fn insert(&mut self, index: usize) -> bool {
        !self.bv.set(index)
    }

    #[inline]
    fn remove(&mut self, index: usize) -> bool {
        self.bv.clear(index)
    }

    #[inline]
    fn rank(&self, index: usize) -> usize {
        self.bv.rank(index) as usize
    }

    #[inline]
    fn count(&self) -> usize {
        self.bv.count_ones()
    }
}
