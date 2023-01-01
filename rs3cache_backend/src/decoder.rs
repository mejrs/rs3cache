//! Functions to decompress cache data.
#![allow(deprecated)]
use std::{
    fmt::{Debug, Display, Formatter},
    io::Read,
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
            let decoder = zlib::Decoder::new(&*data).map_err(DecodeError::ZlibError)?;
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
        [2, _, _, _, _, data @ .., _, _] if let Some(xtea) = xtea => {
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
            let decoder = gzip::Decoder::new(&*data).map_err(DecodeError::GzipError)?;
            let ret = do_read(decoder, length)?;
            Ok(ret)
        }

        // An older variant of the gzip format
        #[cfg(feature = "dat")]
        [b'\x1f', b'\x8b', b'\x08', data @ ..] => {
            if let [data @ .., _version, _version_part2] = data {
                let ret: Result<Bytes, DecodeError> = try {
                    let mut decoder = gzip::Decoder::new(&*data).map_err(DecodeError::GzipError)?;
                    let mut buf = Vec::new();
                    decoder.read_to_end(&mut buf).unwrap();
                    buf.into()
                };
                if ret.is_err() {
                    // Sometimes tools generate caches where trailing versions are missing,
                    // and the below code includes the last two bytes.
                    let data = encoded_data.as_slice();
                    let mut decoder = gzip::Decoder::new(data).map_err(DecodeError::GzipError)?;
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
        [] | [_] | [_, _] | [_, _, _] => Err(DecodeError::Other("File was empty")),

        // Oh no
        _ => unimplemented!("unknown format {:?}", &encoded_data[0..30]),
    }
}

fn do_read(mut decoder: impl Read, len: u32) -> Result<Bytes, DecodeError> {
    if len == 0 {
        return Ok(Bytes::new());
    }
    let mut decoded_data = Vec::with_capacity(len as usize);
    decoder.read_to_end(&mut decoded_data).unwrap();
    Ok(decoded_data.into())
}

#[derive(Debug)]
pub enum DecodeError {
    ZlibError(std::io::Error),
    GzipError(std::io::Error),
    /// Wraps [`bzip2_rs::decoder::DecoderError`].
    BZip2Error(bzip2_rs::decoder::DecoderError),
    #[cfg(feature = "dat2")]
    XteaError,
    Other(&'static str),
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZlibError(io) | Self::GzipError(io) => Display::fmt(&io, f),
            Self::BZip2Error(e) => Display::fmt(&e, f),
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
