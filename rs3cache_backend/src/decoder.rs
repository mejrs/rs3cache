//! Functions to decompress cache data.
#![allow(deprecated)]
use std::{
    fmt::{Debug, Display, Formatter},
    io::{Read, ReadBuf},
    mem::MaybeUninit,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use libflate::{gzip, zlib};

use crate::buf::BufExtra;

/// Decompresses index files.
///
/// Used internally by [`CacheIndex`](crate::index::CacheIndex).
pub fn decompress(mut encoded_data: Vec<u8>, #[cfg(feature = "dat2")] xtea: Option<crate::xtea::Xtea>) -> Result<Bytes, DecodeError> {
    match &mut *encoded_data {
        // The zlib format
        [b'Z', b'L', b'B', b'\x01', x0, x1, x2, x3, data @ ..] => {
            let length = u32::from_be_bytes([*x0, *x1, *x2, *x3]);
            let decoder = zlib::Decoder::new(&*data)?;
            let ret = do_read(decoder, length)?;

            Ok(ret)
        }

        // No compression
        [0, _, _, _, _, data @ .., _, _] => {
            let ret = Bytes::copy_from_slice(data);
            Ok(ret)
        }

        // The bzip format
        [1, _, _, _, _, data @ ..] => {
            let mut header = *b"BZh1";
            let length: &mut [u8; 4] = data.get_mut(0..4).unwrap().try_into().unwrap();
            std::mem::swap(&mut header, length);
            let length = u32::from_be_bytes(header);

            let decoder = bzip2_rs::DecoderReader::new(&*data);
            let ret = do_read(decoder, length)?;
            Ok(ret)
        }

        // A xtea-encrypted gzip
        #[cfg(feature = "dat2")]
        [2, _, _, _, _, data @ .., _, _] if xtea.is_some() => {
            let xtea = xtea.unwrap();
            let decrypted = crate::xtea::Xtea::decrypt(data, xtea);

            if let [x0, x1, x2, x3, decrypted @ ..] = &*decrypted {
                let decoder = gzip::Decoder::new(decrypted).map_err(|_| DecodeError::XteaError)?;
                let length = u32::from_be_bytes([*x0, *x1, *x2, *x3]);
                let ret = do_read(decoder, length)?;
                Ok(ret)
            } else {
                unreachable!()
            }
        }

        // The gzip format
        [2, _y0, _y1, _y2, _y3, x0, x1, x2, x3, data @ ..] => {
            let length = u32::from_be_bytes([*x0, *x1, *x2, *x3]);
            let decoder = gzip::Decoder::new(&*data)?;
            let ret = do_read(decoder, length)?;
            Ok(ret)
        }

        // An older variant of the gzip format
        #[cfg(feature = "dat")]
        [b'\x1f', b'\x8b', b'\x08', data @ ..] => {
            if let [data @ .., _version, _version_part2] = data {
                let ret: Result<Bytes, DecodeError> = try {
                    let mut decoder = gzip::Decoder::new(&*data)?;
                    let mut buf = Vec::new();
                    decoder.read_to_end(&mut buf).unwrap();
                    buf.into()
                };
                if let Err(_) = ret {
                    // Sometimes tools generate caches where trailing versions are missing,
                    // and the below code includes the last two bytes.
                    let data = encoded_data.as_slice();
                    let mut decoder = gzip::Decoder::new(data)?;
                    let mut buf = Vec::new();
                    decoder.read_to_end(&mut buf).unwrap();
                    Ok(buf.into())
                } else {
                    ret
                }
            } else {
                unreachable!()
            }
        }

        // Some tools pack empty files
        [] | [_] | [_, _] | [_, _, _] => Err(DecodeError::Other("File was empty".to_string())),

        // Oh no
        _ => unimplemented!("unknown format {:?}", &encoded_data[0..30]),
    }
}

fn do_read(mut decoder: impl Read, len: u32) -> Result<Bytes, DecodeError> {
    if len == 0 {
        return Ok(Bytes::new());
    }
    let len = len as usize;
    let mut buf = BytesMut::with_capacity(len);

    unsafe {
        let uninit = buf.chunk_mut();
        let uninit = std::slice::from_raw_parts_mut(uninit.as_mut_ptr().cast::<MaybeUninit<u8>>(), uninit.len());
        let mut rbuf = ReadBuf::uninit(uninit);

        decoder.read_buf_exact(&mut rbuf)?;
        let init_len = rbuf.initialized_len();
        buf.set_len(init_len);
    }
    Ok(buf.freeze())
}

#[derive(Debug)]
pub enum DecodeError {
    /// Wraps [`std::io::Error`].
    IoError(std::io::Error),
    /// Wraps [`bzip2_rs::decoder::DecoderError`].
    BZip2Error(bzip2_rs::decoder::DecoderError),
    #[cfg(feature = "dat2")]
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
            #[cfg(feature = "dat2")]
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

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn zlib() -> Result<(), Box<dyn Error>> {
        let file = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/zlib_encoded.dat")).to_vec();
        let buf = decompress(file)?;
        let out = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/zlib_decoded.dat"));
        assert_eq!(&*buf, out);
        Ok(())
    }

    #[test]
    fn bzip() -> Result<(), Box<dyn Error>> {
        let file = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/bzip_encoded.dat")).to_vec();
        let buf = decompress(file)?;
        let out = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/bzip_decoded.dat"));
        assert_eq!(&*buf, out);
        Ok(())
    }

    #[test]
    fn gzip() -> Result<(), Box<dyn Error>> {
        let file = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/gzip_encoded.dat")).to_vec();
        let buf = decompress(file)?;
        let out = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/gzip_decoded.dat"));
        assert_eq!(&*buf, out);
        Ok(())
    }
}
