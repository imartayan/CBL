use crate::compact_int::CompactInt;
use snap::raw::{decompress_len, Decoder, Encoder};

pub trait Compressor {
    fn new() -> Self;
    fn compress_one<const BYTES: usize>(&mut self, x: CompactInt<BYTES>) -> Vec<u8>;
    fn compress_slice<const BYTES: usize>(&mut self, vec: &[CompactInt<BYTES>]) -> Vec<u8>;
    fn decompress_slice<const BYTES: usize>(&mut self, compressed: &[u8])
        -> Vec<CompactInt<BYTES>>;
    fn decompress_len<const BYTES: usize>(&self, compressed: &[u8]) -> usize;
}

#[derive(Debug)]
pub struct SnapCompressor {
    encoder: Encoder,
    decoder: Decoder,
}

impl Compressor for SnapCompressor {
    fn new() -> Self {
        Self {
            encoder: Encoder::new(),
            decoder: Decoder::new(),
        }
    }

    fn compress_one<const BYTES: usize>(&mut self, x: CompactInt<BYTES>) -> Vec<u8> {
        self.encoder.compress_vec(x.bytes()).unwrap()
    }

    fn compress_slice<const BYTES: usize>(&mut self, vec: &[CompactInt<BYTES>]) -> Vec<u8> {
        if vec.is_empty() {
            return Vec::new();
        }
        let raw_vec: Vec<u8> = vec.iter().flat_map(|x| x.bytes()).copied().collect();
        self.encoder.compress_vec(&raw_vec).unwrap()
    }

    fn decompress_slice<const BYTES: usize>(
        &mut self,
        compressed: &[u8],
    ) -> Vec<CompactInt<BYTES>> {
        if compressed.is_empty() {
            return Vec::new();
        }
        let raw_vec = self.decoder.decompress_vec(compressed).unwrap();
        raw_vec.chunks(BYTES).map(CompactInt::from_bytes).collect()
    }

    fn decompress_len<const BYTES: usize>(&self, compressed: &[u8]) -> usize {
        decompress_len(compressed).unwrap() / BYTES
    }
}

impl Default for SnapCompressor {
    fn default() -> Self {
        Self::new()
    }
}
