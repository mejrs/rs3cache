//! Functions to decompress cache data.
#![allow(deprecated)]
use std::{
    fmt::{Debug, Display, Formatter},
    io::Read,
    mem::MaybeUninit,
};

use ::error::Context;
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
            let decoder = zlib::Decoder::new(&*data).context(Zlib)?;
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
        // TODO: see if a missing xtea can be handled more gently
        [2, _, _, _, _, data @ .., _, _] if let Some(xtea) = xtea => {
            let decrypted = crate::xtea::Xtea::decrypt(data, xtea);

            if let [x0, x1, x2, x3, decrypted @ ..] = &*decrypted {
                let decoder = gzip::Decoder::new(decrypted).context(Gzip)?;
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
            let ret = try {
                let decoder = gzip::Decoder::new(&*data).context(Gzip)?;
                let ret = do_read(decoder, length)?;
                ret
            };
            match ret {
                #[cfg(feature = "dat2")]
                Err(_) => Err(Xtea::new()),
                e => e,
            }
        }

        // An older variant of the gzip format
        #[cfg(feature = "dat")]
        [b'\x1f', b'\x8b', b'\x08', data @ ..] => {
            if let [data @ .., _version, _version_part2] = data {
                let ret: Result<Bytes, DecodeError> = try {
                    let mut decoder = gzip::Decoder::new(&*data).context(Gzip)?;
                    let mut buf = Vec::new();
                    decoder.read_to_end(&mut buf).unwrap();
                    buf.into()
                };
                if ret.is_err() {
                    // Sometimes tools generate caches where trailing versions are missing,
                    // and the below code includes the last two bytes.
                    let data = encoded_data.as_slice();
                    let mut decoder = gzip::Decoder::new(data).context(Gzip)?;
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
        [] | [_] | [_, _] | [_, _, _] => Err(Empty::new(encoded_data)),

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

#[derive(::error::Error)]
pub enum DecodeError {
    #[error = "could not decompress zlib-compressed buffer"]
    Zlib {
        #[source]
        source: std::io::Error,
    },
    #[error = "could not decompress gzib-compressed buffer"]
    Gzip {
        #[source]
        source: std::io::Error,
    },
    #[error = "could not decompress bzip2-compressed buffer"]
    BZip2 {
        #[source]
        source: bzip2_rs::decoder::DecoderError,
    },
    #[error = "passed empty buffer: {buf:?}"]
    Empty { buf: Vec<u8> },
    #[error = "decoding format not implemented"]
    Unimplemented { buf: Vec<u8> },
    #[cfg(feature = "dat2")]
    #[error = "xtea was not found"]
    Xtea,
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
