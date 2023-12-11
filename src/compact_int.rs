// https://github.com/Daniel-Liu-c0deb0t/simple-saca/blob/main/src/compact_vec.rs

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CompactInt<const BYTES: usize>([u8; BYTES]);

impl<const BYTES: usize> CompactInt<BYTES> {
    #[inline(always)]
    pub fn get_usize(&self) -> usize {
        let mut res = 0u64;
        unsafe {
            std::ptr::copy_nonoverlapping(self.0.as_ptr(), &mut res as *mut _ as _, BYTES);
        }
        res as usize
    }

    #[inline(always)]
    pub fn set_usize(&mut self, val: usize) {
        unsafe {
            std::ptr::copy_nonoverlapping(&val as *const _ as _, self.0.as_mut_ptr(), BYTES);
        }
    }
}
