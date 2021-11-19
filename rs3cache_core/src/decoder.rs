//! Functions to decompress cache data.
#![allow(deprecated)]
use std::{
    fmt::{Debug, Display, Formatter},
    io::Read,
};

use bytes::{Buf, Bytes};
use libflate::{gzip, zlib};

use crate::buf::BufExtra;
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
    pub const ZLIB: &'static [u8] = b"ZLB";
}

/// Decompresses index files.
///
/// Used internally by [`CacheIndex`](crate::index::CacheIndex).
#[track_caller]
pub fn decompress(
    encoded_data: Vec<u8>,
    filesize: Option<u32>,
    #[cfg(feature = "osrs")] xtea: Option<crate::xtea::Xtea>,
) -> Result<Bytes, DecodeError> {
    match &encoded_data[0..3] {
        Compression::ZLIB => {
            let mut decoder = zlib::Decoder::new(&encoded_data[8..])?;
            let mut decoded_data = Vec::with_capacity(filesize.unwrap_or(0) as usize);
            decoder.read_to_end(&mut decoded_data)?;
            Ok(decoded_data.into())
        }

        &[Compression::NONE, ..] => {
            // length is encoded_data[1..5] as u32 + 7
            Ok(encoded_data[5..(encoded_data.len() - 2)].to_vec().into())
        }

        &[Compression::BZIP, ..] => {
            let mut temp = b"BZh1".to_vec();
            let length = u32::from_be_bytes([encoded_data[5], encoded_data[6], encoded_data[7], encoded_data[8]]) as usize;

            let mut decoded_data = Vec::with_capacity(filesize.unwrap_or(length as u32) as usize);

            temp.extend(&encoded_data[9..]);

            let mut decoder = bzip2_rs::DecoderReader::new(temp.as_slice());

            decoder.read_to_end(&mut decoded_data)?;
            Ok(decoded_data.into())
        }

        #[cfg(feature = "osrs")]
        &[Compression::GZIP, ..] if xtea.is_some() => {
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

            Ok(decoded_data.into())
        }

        &[Compression::GZIP, ..] => {
            let mut decoder = gzip::Decoder::new(&encoded_data[9..])?;
            let mut decoded_data = Vec::with_capacity(filesize.unwrap_or(0) as usize);
            decoder.read_to_end(&mut decoded_data)?;
            Ok(decoded_data.into())
        }

        other => unimplemented!("unknown format {:?}", other),
    }
}

#[derive(Debug)]
pub enum DecodeError {
    /// Wraps [`std::io::Error`].
    IoError(std::io::Error),
    /// Wraps [`bzip2_rs::decoder::DecoderError`].
    BZip2Error(bzip2_rs::decoder::DecoderError),
    #[cfg(feature = "osrs")]
    XteaError,
    Other(String),
}

impl From<std::io::Error> for DecodeError {
    fn from(cause: std::io::Error) -> Self {
        Self::IoError(cause)
    }
}

impl From<bzip2_rs::decoder::DecoderError> for DecodeError {
    fn from(cause: bzip2_rs::decoder::DecoderError) -> Self {
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
