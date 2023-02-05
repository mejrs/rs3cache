//! Wrapper around [`Cursor`](std::io::Cursor).
//!
//! This module provides various reads used to decode the cache data
use core::ops::Deref;
use std::{
    fmt::{self, Debug, Display, Formatter},
    io::{prelude::*, Cursor, SeekFrom},
    iter,
    panic::Location,
};

use bytes::{Buf, Bytes};
use serde::{Serialize, Serializer};

use crate::error::{CacheError, Context};

#[derive(Debug, Clone)]
pub struct ReadError {
    location: &'static Location<'static>,
    kind: Kind,
}
impl ReadError {
    #[track_caller]
    pub fn eof() -> Self {
        Self {
            location: Location::caller(),
            kind: Kind::Error(ReadErrorKind::Eof),
        }
    }
    #[track_caller]
    pub fn not_nul_terminated() -> Self {
        Self {
            location: Location::caller(),
            kind: Kind::Error(ReadErrorKind::NotNulTerminated),
        }
    }
    #[track_caller]
    pub fn opcode_not_implemented(opcode: u8) -> Self {
        Self {
            location: Location::caller(),
            kind: Kind::Error(ReadErrorKind::OpcodeNotImplemented(opcode)),
        }
    }

    #[track_caller]
    pub fn not_exhausted() -> Self {
        Self {
            location: Location::caller(),
            kind: Kind::Error(ReadErrorKind::NotExhausted),
        }
    }

    #[cfg(debug_assertions)]
    #[track_caller]
    pub fn duplicate_opcode(opcodes: Vec<u8>, opcode: u8) -> Self {
        Self {
            location: Location::caller(),
            kind: Kind::Error(ReadErrorKind::DuplicateOpcode(opcodes, opcode)),
        }
    }

    #[track_caller]
    pub fn add_context(self, ctx: String) -> Self {
        Self {
            location: Location::caller(),
            kind: Kind::Bubbled(ctx, Box::new(self)),
        }
    }

    #[track_caller]
    pub fn add_context_id(self, ctx: u32) -> Self {
        Self {
            location: Location::caller(),
            kind: Kind::ContextId(ctx, Box::new(self)),
        }
    }

    pub fn add_decode_context(self, #[cfg(debug_assertions)] opcodes: Vec<u8>, remainder: Bytes, parsed: String) -> Self {
        Self {
            location: self.location,
            kind: Kind::DecodeContext(
                #[cfg(debug_assertions)]
                opcodes,
                remainder,
                parsed,
                Box::new(self),
            ),
        }
    }
}

#[derive(Debug, Clone)]
enum Kind {
    Error(ReadErrorKind),
    ContextId(u32, Box<ReadError>),
    Bubbled(String, Box<ReadError>),
    DecodeContext(#[cfg(debug_assertions)] Vec<u8>, Bytes, String, Box<ReadError>),
}

#[derive(Debug, Clone)]
pub enum ReadErrorKind {
    Eof,
    NotNulTerminated,
    NotExhausted,
    OpcodeNotImplemented(u8),
    #[cfg(debug_assertions)]
    DuplicateOpcode(Vec<u8>, u8),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Kind::*;
        use ReadErrorKind::*;

        let location = self.location;

        match &self.kind {
            Error(Eof) => writeln!(f, "Unexpected end of file ({location})")?,
            Error(NotNulTerminated) => writeln!(f, "Buffer did not contain nul terminator")?,
            Error(OpcodeNotImplemented(opcode)) => {
                writeln!(f, "Read opcode {opcode}, but decoding opcode {opcode} is not implemented. ({location})")?
            }
            Error(NotExhausted) => writeln!(f, "Reached terminating opcode but the buffer was not exhausted ({location})")?,
            #[cfg(debug_assertions)]
            Error(DuplicateOpcode(_, opcode)) => writeln!(f, "Read opcode {opcode}, but opcode {opcode} was already decoded. ({location})")?,
            ContextId(id, _) => writeln!(f, "Could not decode id {id} ({location})")?,
            Bubbled(ref ctx, _) => writeln!(f, "Could not decode {ctx}")?,
            #[cfg(debug_assertions)]
            DecodeContext(opcodes, remainder, parsed, src) => {
                writeln!(f, "{src}")?;
                writeln!(f, "Note: The unread remainder of the buffer consists of {remainder:?}")?;
                writeln!(f)?;
                writeln!(f, "Note: The opcodes read were {opcodes:?}")?;
                writeln!(f)?;
                writeln!(f, "Note: Managed to read up to:")?;
                writeln!(f, "{parsed}")?;
            }
            #[cfg(not(debug_assertions))]
            DecodeContext(remainder, parsed, src) => {
                writeln!(f, "{src}")?;
                writeln!(f, "Note: The unread remainder of the buffer consists of {:?}", remainder)?;
                writeln!(f)?;
                writeln!(f, "Note: Managed to read up to:")?;
                writeln!(f, "{parsed}")?;
            }
        };

        Ok(())
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.kind {
            Kind::ContextId(_, ref src) => Some(src),
            Kind::Bubbled(_, ref src) => Some(src),
            _ => None,
        }
    }
}

pub trait BufExtra: Buf + Sized + Clone {
    #[track_caller]
    #[inline]
    fn try_get_u8(&mut self) -> Result<u8, ReadError> {
        if self.remaining() >= 1 {
            Ok(self.get_u8())
        } else {
            Err(ReadError::eof())
        }
    }
    #[track_caller]
    #[inline]
    fn try_get_i8(&mut self) -> Result<i8, ReadError> {
        if self.remaining() >= 1 {
            Ok(self.get_i8())
        } else {
            Err(ReadError::eof())
        }
    }
    #[track_caller]
    #[inline]
    fn try_get_u16(&mut self) -> Result<u16, ReadError> {
        if self.remaining() >= 2 {
            Ok(self.get_u16())
        } else {
            Err(ReadError::eof())
        }
    }

    #[track_caller]
    #[inline]
    fn try_get_i32(&mut self) -> Result<i32, ReadError> {
        if self.remaining() >= 4 {
            Ok(self.get_i32())
        } else {
            Err(ReadError::eof())
        }
    }
    #[track_caller]
    #[inline]
    fn try_get_u32(&mut self) -> Result<u32, ReadError> {
        if self.remaining() >= 4 {
            Ok(self.get_u32())
        } else {
            Err(ReadError::eof())
        }
    }

    #[track_caller]
    #[inline]
    fn try_get_uint(&mut self, nbytes: usize) -> Result<u64, ReadError> {
        if self.remaining() >= nbytes {
            Ok(self.get_uint(nbytes))
        } else {
            Err(ReadError::eof())
        }
    }
    #[inline]
    fn get_array<const LENGTH: usize>(&mut self) -> [u8; LENGTH] {
        let mut dst = [0; LENGTH];
        self.copy_to_slice(&mut dst);
        dst
    }
    #[inline]
    fn try_get_array<const LENGTH: usize>(&mut self) -> Result<[u8; LENGTH], ReadError> {
        if self.remaining() >= LENGTH {
            Err(ReadError::eof())
        } else {
            let mut dst = [0; LENGTH];
            self.copy_to_slice(&mut dst);
            Ok(dst)
        }
    }

    /// Reads two or four unsigned bytes as an 32-bit unsigned integer.
    #[track_caller]
    #[inline]
    fn try_get_smart32(&mut self) -> Result<Option<u32>, ReadError> {
        let condition = self.chunk().first().ok_or_else(ReadError::eof)? & 0x80 == 0x80;

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
        let condition = self.chunk()[0] & 0x80 == 0x80;

        if condition {
            Some(self.get_u32() & 0x7FFFFFFF)
        } else {
            let value = self.get_u16() as u32;
            if value == 0x7FFF {
                None
            } else {
                Some(value)
            }
        }
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
        const TERMINATOR: u8 = if cfg!(feature = "dat") { b'\n' } else { b'\0' };

        let chunk = self.chunk();
        let nul_pos = memchr::memchr(TERMINATOR, chunk).ok_or_else(ReadError::not_nul_terminated)?;
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

#[derive(Clone, Debug)]
pub enum JStringKind<R: Buf> {
    Refcounted { buf: R, len: usize },
    Allocated(String),
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
