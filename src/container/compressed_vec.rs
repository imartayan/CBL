use super::{CompressedContainer, Compressor, Container};
use crate::compact_int::CompactInt;
use core::marker::PhantomData;
use core::mem::swap;

#[derive(Debug)]
enum MaybeCompressed<T, C: Container<T>> {
    Plain(C, PhantomData<T>),
    Compressed(Vec<u8>),
}

#[derive(Debug)]
pub struct SemiCompressed<
    const BYTES: usize,
    const THRESHOLD: usize,
    ContainerT: Container<CompactInt<BYTES>>,
    CompressorT: Compressor,
> {
    data: MaybeCompressed<CompactInt<BYTES>, ContainerT>,
    _marker: PhantomData<CompressorT>,
}

impl<
        const BYTES: usize,
        const THRESHOLD: usize,
        ContainerT: Container<CompactInt<BYTES>>,
        CompressorT: Compressor,
    > CompressedContainer<BYTES, CompressorT>
    for SemiCompressed<BYTES, THRESHOLD, ContainerT, CompressorT>
{
    #[inline]
    fn new() -> Self {
        Self {
            data: MaybeCompressed::Plain(ContainerT::new(), PhantomData),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn new_with_one(_compressor: &mut CompressorT, x: CompactInt<BYTES>) -> Self {
        Self {
            data: MaybeCompressed::Plain(ContainerT::new_with_one(x), PhantomData),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn len(&self, compressor: &CompressorT) -> usize {
        match &self.data {
            MaybeCompressed::Plain(container, _) => container.len(),
            MaybeCompressed::Compressed(compressed) => {
                compressor.decompress_len::<BYTES>(compressed)
            }
        }
    }

    #[inline]
    fn contains(&self, compressor: &mut CompressorT, x: &CompactInt<BYTES>) -> bool {
        match &self.data {
            MaybeCompressed::Plain(container, _) => container.contains(x),
            MaybeCompressed::Compressed(compressed) => {
                let vec = compressor.decompress_slice(compressed);
                let container = unsafe { ContainerT::from_vec_unchecked(vec) };
                container.contains(x)
            }
        }
    }

    #[inline]
    fn insert(&mut self, compressor: &mut CompressorT, x: CompactInt<BYTES>) -> bool {
        match &mut self.data {
            MaybeCompressed::Plain(container, _) => {
                let res = container.insert(x);
                if container.len() >= THRESHOLD {
                    let mut swapped_container = ContainerT::new();
                    swap(container, &mut swapped_container);
                    self.data = MaybeCompressed::Compressed(
                        compressor.compress_slice(&swapped_container.to_vec()),
                    )
                }
                res
            }
            MaybeCompressed::Compressed(compressed) => {
                let vec = compressor.decompress_slice(compressed);
                let mut container = unsafe { ContainerT::from_vec_unchecked(vec) };
                let res = container.insert(x);
                self.data =
                    MaybeCompressed::Compressed(compressor.compress_slice(&container.to_vec()));
                res
            }
        }
    }

    #[inline]
    fn remove(&mut self, compressor: &mut CompressorT, x: &CompactInt<BYTES>) -> bool {
        match &mut self.data {
            MaybeCompressed::Plain(container, _) => container.remove(x),
            MaybeCompressed::Compressed(compressed) => {
                let vec = compressor.decompress_slice(compressed);
                let mut container = unsafe { ContainerT::from_vec_unchecked(vec) };
                let res = container.remove(x);
                if container.len() < THRESHOLD {
                    self.data = MaybeCompressed::Plain(container, PhantomData);
                } else {
                    self.data =
                        MaybeCompressed::Compressed(compressor.compress_slice(&container.to_vec()));
                }
                res
            }
        }
    }

    #[inline]
    fn insert_iter<I: ExactSizeIterator<Item = CompactInt<BYTES>>>(
        &mut self,
        compressor: &mut CompressorT,
        it: I,
    ) {
        match &mut self.data {
            MaybeCompressed::Plain(container, _) => {
                container.insert_iter(it);
                if container.len() >= THRESHOLD {
                    let mut swapped_container = ContainerT::new();
                    swap(container, &mut swapped_container);
                    self.data = MaybeCompressed::Compressed(
                        compressor.compress_slice(&swapped_container.to_vec()),
                    )
                }
            }
            MaybeCompressed::Compressed(compressed) => {
                let vec = compressor.decompress_slice(compressed);
                let mut container = unsafe { ContainerT::from_vec_unchecked(vec) };
                container.insert_iter(it);
                self.data =
                    MaybeCompressed::Compressed(compressor.compress_slice(&container.to_vec()));
            }
        }
    }

    #[inline]
    fn remove_iter<I: ExactSizeIterator<Item = CompactInt<BYTES>>>(
        &mut self,
        compressor: &mut CompressorT,
        it: I,
    ) {
        match &mut self.data {
            MaybeCompressed::Plain(container, _) => container.remove_iter(it),
            MaybeCompressed::Compressed(compressed) => {
                let vec = compressor.decompress_slice(compressed);
                let mut container = unsafe { ContainerT::from_vec_unchecked(vec) };
                container.remove_iter(it);
                self.data =
                    MaybeCompressed::Compressed(compressor.compress_slice(&container.to_vec()));
            }
        }
    }

    // #[inline]
    // fn iter<'a>(
    //     &'a self,
    //     compressor: &mut CompressorT,
    // ) -> impl ExactSizeIterator<Item = &'a CompactInt<BYTES>> {
    //     match &self.data {
    //         MaybeCompressed::Plain(container, _) => container.iter(),
    //         MaybeCompressed::Compressed(compressed) => {
    //             let vec = compressor.decompress_slice(compressed);
    //             let container = unsafe { ContainerT::from_vec_unchecked(vec) };
    //             container.iter()
    //         }
    //     }
    // }
}
