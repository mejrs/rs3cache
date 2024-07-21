//! Wrapper around [`Cursor`](std::io::Cursor).
//!
//! This module provides various reads used to decode the cache data
use core::ops::Deref;
use std::{fmt::Debug, panic::Location};

use ::error::Context;
use bytes::{Buf, Bytes};
use serde::{Serialize, Serializer};

#[derive(::error::Error)]
pub enum ReadError {
    #[error = "reached the end of the buffer unexpectedly; expected at least {expected} more bytes but only {found} were available"]
    Eof {
        #[location]
        location: &'static Location<'static>,
        expected: usize,
        found: usize,
    },
    #[error = "the buffer did not contain a {terminator} terminator"]
    NotNulTerminated {
        #[location]
        location: &'static Location<'static>,
        terminator: u8,
    },
    #[error = "the buffer was not exhausted: {remainder:?} left."]
    NotExhausted {
        #[location]
        location: &'static Location<'static>,
        remainder: Bytes,
    },
    #[error = "opcode {opcode} is not implemented"]
    OpcodeNotImplemented {
        opcode: u8,
        #[location]
        location: &'static Location<'static>,
    },
    #[error = "could not parse buffer"]
    #[cfg_attr(debug_assertions, help = "managed to read up to {thing}")]
    #[cfg_attr(debug_assertions, help = "managed to decode opcodes {opcodes:?}")]
    #[help = "the remainder of the buffer is {buffer:?}"]
    WithInfo {
        #[source]
        source: Box<ReadError>,
        #[cfg(debug_assertions)]
        opcodes: Vec<u8>,
        buffer: Bytes,
        #[cfg(debug_assertions)]
        thing: String,
    },
    #[error = "reached the end of the file unexpectedly"]
    FileSeek {
        #[source]
        source: std::io::Error,
        #[location]
        location: &'static Location<'static>,
    },
}

pub trait BufExtra: Buf + Sized + Clone {
    #[track_caller]
    #[inline]
    fn try_get_u8(&mut self) -> Result<u8, ReadError> {
        if self.remaining() >= 1 {
            Ok(self.get_u8())
        } else {
            Err(Eof::new(1, self.remaining()))
        }
    }
    #[track_caller]
    #[inline]
    fn try_get_i8(&mut self) -> Result<i8, ReadError> {
        if self.remaining() >= 1 {
            Ok(self.get_i8())
        } else {
            Err(Eof::new(1, self.remaining()))
        }
    }
    #[track_caller]
    #[inline]
    fn try_get_u16(&mut self) -> Result<u16, ReadError> {
        if self.remaining() >= 2 {
            Ok(self.get_u16())
        } else {
            Err(Eof::new(2, self.remaining()))
        }
    }

    #[track_caller]
    #[inline]
    fn try_get_i32(&mut self) -> Result<i32, ReadError> {
        if self.remaining() >= 4 {
            Ok(self.get_i32())
        } else {
            Err(Eof::new(4, self.remaining()))
        }
    }
    #[track_caller]
    #[inline]
    fn try_get_u32(&mut self) -> Result<u32, ReadError> {
        if self.remaining() >= 4 {
            Ok(self.get_u32())
        } else {
            Err(Eof::new(4, self.remaining()))
        }
    }

    #[track_caller]
    #[inline]
    fn try_get_uint(&mut self, nbytes: usize) -> Result<u64, ReadError> {
        if self.remaining() >= nbytes {
            Ok(self.get_uint(nbytes))
        } else {
            Err(Eof::new(nbytes, self.remaining()))
        }
    }
    #[inline]
    fn get_array<const LENGTH: usize>(&mut self) -> [u8; LENGTH] {
        self.try_get_array().unwrap()
    }
    #[inline]
    fn try_get_array<const LENGTH: usize>(&mut self) -> Result<[u8; LENGTH], ReadError> {
        if self.remaining() >= LENGTH {
            let mut dst = [0; LENGTH];
            self.copy_to_slice(&mut dst);
            Ok(dst)
        } else {
            Err(Eof::new(LENGTH, self.remaining()))
        }
    }

    /// Reads two or four unsigned bytes as an 32-bit unsigned integer.
    #[track_caller]
    #[inline]
    fn try_get_smart32(&mut self) -> Result<Option<u32>, ReadError> {
        let condition = self.chunk().first().context(Eof {
            expected: 1,
            found: self.remaining(),
        })? & 0x80
            == 0x80;

        let ret = if condition {
            Some(self.try_get_u32()? & 0x7FFFFFFF)
        } else {
            let value = self.try_get_u16()? as u32;
            if value == 0x7FFF {
                None
            } else {
                Some(value)
            }
        };
        Ok(ret)
    }

    /// Reads two or four unsigned bytes as an 32-bit unsigned integer.
    #[inline]
    fn get_smart32(&mut self) -> Option<u32> {
        self.try_get_smart32().unwrap()
    }

    /// Reads one or two unsigned bytes as an 16-bit unsigned integer.
    #[inline]
    fn try_get_unsigned_smart(&mut self) -> Result<u16, ReadError> {
        let mut i = self.try_get_u8()? as u16;
        let ret = if i >= 0x80 {
            i -= 0x80;
            i << 8 | (self.try_get_u8()? as u16)
        } else {
            i
        };
        Ok(ret)
    }

    /// Reads one or two unsigned bytes as an 16-bit unsigned integer.
    #[inline]
    fn get_unsigned_smart(&mut self) -> u16 {
        let mut i = self.get_u8() as u16;
        if i >= 0x80 {
            i -= 0x80;
            i << 8 | (self.get_u8() as u16)
        } else {
            i
        }
    }

    /// Reads Kind one or two bytes.
    #[inline]
    fn get_decr_smart(&mut self) -> Option<u16> {
        match self.get_u8() as u16 {
            first if first < 128 => first.checked_sub(1),
            first => (first << 8 | self.get_u8() as u16).checked_sub(0x8000).unwrap().checked_sub(1),
        }
    }

    /// Reads masked data.
    #[inline]
    fn get_masked_data(&mut self) -> Vec<(Option<u32>, Option<u32>)> {
        let mut result = Vec::new();
        let mut mask = self.get_u8();
        while mask > 0 {
            if mask & 0x1 == 1 {
                result.push((self.get_smart32(), self.get_decr_smart().map(|c| c as u32)));
            } else {
                result.push((None, None));
            }
            mask /= 2;
        }
        result
    }

    /// Reads a multiple of two bytes as an 32-bit unsigned integer.
    #[inline]
    fn get_smarts(&mut self) -> u32 {
        let mut value: u32 = 0;
        loop {
            match self.get_unsigned_smart() as u32 {
                0x7FFF => value = value.checked_add(0x7FFF).expect("Detected u32 overflow in buffer.get_smarts()"),
                offset => break value.checked_add(offset).expect("Detected u32 overflow in buffer.get_smarts()"),
            }
        }
    }

    /// Reads one byte, returning 8 boolean bitflags.
    #[inline]
    fn get_bitflags(&mut self) -> [bool; 8] {
        let flags = self.get_u8();
        [
            flags & 0x1 != 0,
            flags & 0x2 != 0,
            flags & 0x4 != 0,
            flags & 0x8 != 0,
            flags & 0x10 != 0,
            flags & 0x20 != 0,
            flags & 0x40 != 0,
            flags & 0x80 != 0,
        ]
    }

    /// Reads a 0-terminated String from the buffer
    #[inline]
    fn try_get_string(&mut self) -> Result<JString<Self>, ReadError> {
        let terminator: u8 = if cfg!(feature = "dat") { b'\n' } else { b'\0' };

        let chunk = self.chunk();
        let nul_pos = memchr::memchr(terminator, chunk).context(NotNulTerminated { terminator })?;
        let chunk = unsafe { chunk.get_unchecked(0..nul_pos) };

        let s = if std::str::from_utf8(chunk).is_ok() {
            // SAFETY: We just checked that it's valid utf8
            unsafe { JString::new(self.clone(), nul_pos) }
        } else {
            // this string format is not utf8, of course :)
            chunk.iter().map(|&i| i as char).collect::<String>().into()
        };
        self.advance(nul_pos + 1);
        Ok(s)
    }

    /// Reads a 0-terminated String from the buffer
    #[inline]
    fn get_string(&mut self) -> JString<Self> {
        self.try_get_string().expect("terminator not found")
    }

    /// Reads a 0-start and 0-terminated String from the buffer.
    #[inline]
    fn get_padded_string(&mut self) -> JString<Self> {
        self.get_u8();
        self.get_string()
    }

    /// Reads three unsigned bytes , returning a `[red, blue, green]` array.
    #[inline]
    fn get_rgb(&mut self) -> [u8; 3] {
        self.get_array()
    }

    /// Reads two obfuscated bytes.
    #[inline]
    fn try_get_masked_index(&mut self) -> Result<u16, ReadError> {
        // big TODO
        self.try_get_u16()
    }

    /// Reads two obfuscated bytes.
    #[inline]
    fn get_masked_index(&mut self) -> u16 {
        // big TODO
        self.get_u16()
    }
}

impl<T: Buf + Clone> BufExtra for T {}

#[derive(Clone, Debug)]
pub struct JString<R: Buf> {
    inner: JStringKind<R>,
}

impl<R: Buf> Serialize for JString<R> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self)
    }
}

#[derive(Clone)]
pub enum JStringKind<R: Buf> {
    Refcounted { buf: R, len: usize },
    Allocated(String),
}

impl<R: ::core::fmt::Debug + Buf> ::core::fmt::Debug for JStringKind<R> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            JStringKind::Refcounted { buf, len } => f
                .debug_struct("Refcounted")
                .field("buf", &&buf.chunk()[..*len])
                .field("len", len)
                .finish(),
            JStringKind::Allocated(s) => f.debug_tuple("Allocated").field(s).finish(),
        }
    }
}

impl<R: Buf> JString<R> {
    /// # Safety
    ///
    /// buf[0..len] must be in-bounds and be valid utf8.
    pub unsafe fn new(buf: R, len: usize) -> Self {
        Self {
            inner: JStringKind::Refcounted { buf, len },
        }
    }
}

impl<R: Buf> From<String> for JString<R> {
    fn from(s: String) -> Self {
        Self {
            inner: JStringKind::Allocated(s),
        }
    }
}

impl<R: Buf> AsRef<str> for JString<R> {
    fn as_ref(&self) -> &str {
        match &self.inner {
            JStringKind::Refcounted { buf, len } => unsafe { std::str::from_utf8_unchecked(buf.chunk().get_unchecked(0..*len)) },
            JStringKind::Allocated(s) => s.as_str(),
        }
    }
}

impl<R: Buf> Deref for JString<R> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<R: Buf> PartialEq for JString<R> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<R: Buf> PartialEq<str> for JString<R> {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

impl<R: Buf> Eq for JString<R> {}

#[cfg(feature = "pyo3")]
impl<R: Buf> pyo3::IntoPy<pyo3::Py<pyo3::PyAny>> for JString<R> {
    fn into_py(self, py: pyo3::Python<'_>) -> pyo3::Py<pyo3::PyAny> {
        pyo3::types::PyString::new(py, &self).into()
    }
}
