pub struct TinyBitvector([u64; 4]);

impl TinyBitvector {
    #[inline(always)]
    pub fn new() -> Self {
        Self([0u64; 4])
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        (self.0[0].count_ones()
            + self.0[1].count_ones()
            + self.0[2].count_ones()
            + self.0[3].count_ones()) as usize
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    #[inline(always)]
    pub fn contains(&self, index: u8) -> bool {
        self.0[index as usize / 64] & (1 << (index as u64 % 64)) != 0
    }

    #[inline]
    pub fn insert(&mut self, index: u8) -> bool {
        let old = self.0[index as usize / 64];
        self.0[index as usize / 64] = old | (1 << (index as u64 % 64));
        self.0[index as usize / 64] != old
    }

    #[inline]
    pub fn remove(&mut self, index: u8) -> bool {
        let old = self.0[index as usize / 64];
        self.0[index as usize / 64] = old & !(1 << (index as u64 % 64));
        self.0[index as usize / 64] != old
    }

    #[inline]
    pub fn rank(&self, index: u8) -> usize {
        let rank = (self.0[index as usize / 64] & ((1 << (index as u64 % 64)) - 1)).count_ones();
        match index / 64 {
            0 => rank as usize,
            1 => (rank + self.0[0].count_ones()) as usize,
            2 => (rank + self.0[0].count_ones() + self.0[1].count_ones()) as usize,
            3 => {
                (rank + self.0[0].count_ones() + self.0[1].count_ones() + self.0[2].count_ones())
                    as usize
            }
            _ => unreachable!(),
        }
    }
}
