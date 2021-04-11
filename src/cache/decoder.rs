//! Functions to decompress cache data.

use crate::utils::error::{CacheError, CacheResult};
use libflate::{gzip, zlib};
use std::io::Read;

/// Enumeration of different compression types.
pub struct Compression;

impl Compression {
    /// Token for no compression.
    pub const NONE: u8 = 0;
    /// Token for bzip compression.
    pub const BZIP: u8 = 1;
    /// Token for gzip compression.
    pub const GZIP: u8 = 2;
    /// Token for zlib compression.
    pub const ZLIB: &'static [u8; 3] = b"ZLB";
}

/// Decompresses index files.
///
/// Used internally by [`CacheIndex`](crate::cache::index::CacheIndex).
pub fn decompress(encoded_data: Vec<u8>, filesize: Option<u32>) -> CacheResult<Vec<u8>> {
    if &encoded_data[0..3] == Compression::ZLIB {
        let mut decoder = zlib::Decoder::new(&encoded_data[8..])?;
        let mut decoded_data = Vec::with_capacity(filesize.unwrap_or(0) as usize);
        decoder.read_to_end(&mut decoded_data)?;
        Ok(decoded_data)
    } else if encoded_data[0] == Compression::NONE {
        // length is encoded_data[1..5] as u32 + 7
        Ok(encoded_data[5..(encoded_data.len() - 2)].to_vec())
    } else if encoded_data[0] == Compression::BZIP {
        let mut temp = b"BZh1".to_vec();
        temp.extend(&encoded_data[9..]);

        let mut decoded_data = Vec::with_capacity(
            filesize.ok_or_else(|| CacheError::DecompressionError("bzip2 length must be known".to_string(), std::backtrace::Backtrace::capture()))?
                as usize,
        );

        let mut decoder = bzip2::Decompress::new(false);

        decoder.decompress_vec(&temp, &mut decoded_data)?;

        Ok(decoded_data)
    } else if encoded_data[0] == Compression::GZIP {
        let mut decoder = gzip::Decoder::new(&encoded_data[9..])?;
        let mut decoded_data = Vec::with_capacity(filesize.unwrap_or(0) as usize);
        decoder.read_to_end(&mut decoded_data)?;
        Ok(decoded_data)
    } else {
        Err(CacheError::DecompressionError(
            format!("Unknown compression format: {:?}", &encoded_data[0..10]),
            std::backtrace::Backtrace::capture(),
        ))
    }
}
