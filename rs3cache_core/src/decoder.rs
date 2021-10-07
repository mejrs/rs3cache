//! Functions to decompress cache data.

use std::{
    fmt::{Debug, Display, Formatter},
    io::Read,
};

use libflate::{gzip, zlib};

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
/// Used internally by [`CacheIndex`](crate::index::CacheIndex).
#[cfg(feature = "rs3")]
pub fn decompress(encoded_data: Vec<u8>, filesize: Option<u32>) -> Result<Vec<u8>, DecodeError> {
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

        let mut decoded_data = Vec::with_capacity(filesize.ok_or_else(|| DecodeError::Other("bzip2 length must be known".to_string()))? as usize);

        let mut decoder = bzip2::Decompress::new(false);

        decoder.decompress_vec(&temp, &mut decoded_data)?;

        Ok(decoded_data)
    } else if encoded_data[0] == Compression::GZIP {
        let mut decoder = gzip::Decoder::new(&encoded_data[9..])?;
        let mut decoded_data = Vec::with_capacity(filesize.unwrap_or(0) as usize);
        decoder.read_to_end(&mut decoded_data)?;
        Ok(decoded_data)
    } else {
        unimplemented!("unknown format {}", encoded_data[0])
    }
}

#[cfg(feature = "osrs")]
pub fn decompress(encoded_data: Vec<u8>, filesize: Option<u32>, xtea: Option<crate::xtea::Xtea>) -> Result<Vec<u8>, DecodeError> {
    use crate::buf::Buffer;

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
        let mut length = Buffer::new(&encoded_data[5..9]);
        let length = length.read_int();
        temp.extend(&encoded_data[9..]);

        let mut decoded_data = Vec::with_capacity(length as _);

        let mut decoder = bzip2::Decompress::new(false);

        decoder.decompress_vec(&temp, &mut decoded_data)?;
        Ok(decoded_data)
    } else if xtea.is_some() && encoded_data[0] == Compression::GZIP {
        let length = u32::from_be_bytes([encoded_data[1], encoded_data[2], encoded_data[3], encoded_data[4]]) as usize;

        let xtea = xtea.unwrap();
        let decrypted = crate::xtea::Xtea::decrypt(&encoded_data[5..(length + 9)], xtea);

        let mut decoder = match gzip::Decoder::new(&decrypted[4..]) {
            Ok(decoder) => decoder,
            Err(_e) => {
                return Err(DecodeError::XteaError);
            }
        };
        let mut decoded_data = Vec::with_capacity(filesize.unwrap_or(0) as usize);
        decoder.read_to_end(&mut decoded_data).expect("oops");

        Ok(decoded_data)
    } else if encoded_data[0] == Compression::GZIP {
        let mut decoder = gzip::Decoder::new(&encoded_data[9..])?;
        let mut decoded_data = Vec::with_capacity(filesize.unwrap_or(0) as usize);
        decoder.read_to_end(&mut decoded_data)?;
        Ok(decoded_data)
    } else {
        unimplemented!("unknown format {}", encoded_data[0])
    }
}

#[derive(Debug)]
pub enum DecodeError {
    /// Wraps [`bzip2::Error`].
    IoError(std::io::Error),
    BZip2Error(bzip2::Error),
    #[cfg(feature = "osrs")]
    XteaError,
    Other(String),
}

impl From<std::io::Error> for DecodeError {
    fn from(cause: std::io::Error) -> Self {
        Self::IoError(cause)
    }
}

impl From<bzip2::Error> for DecodeError {
    fn from(cause: bzip2::Error) -> Self {
        Self::BZip2Error(cause)
    }
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BZip2Error(e) => Display::fmt(&e, f),
            Self::IoError(e) => Display::fmt(&e, f),
            Self::Other(e) => Display::fmt(&e, f),
            #[cfg(feature = "osrs")]
            Self::XteaError => Display::fmt("XteaError", f),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BZip2Error(e) => Some(e),
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}
