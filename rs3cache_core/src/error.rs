use std::{backtrace::Backtrace, borrow::Cow, path::PathBuf};

use path_absolutize::*;

use crate::decoder::DecodeError;
/// Result wrapper for [`CacheError`].
pub type CacheResult<T> = Result<T, CacheError>;

/// An error type for things that can go wrong when reading from the cache.
pub enum CacheError {
    /// Wraps [`sqlite::Error`].
    #[cfg(feature = "rs3")]
    SqliteError(sqlite::Error),
    DecodeError(DecodeError),
    /// Wraps [`io.error`](std::io::Error).
    IoError(std::io::Error),
    /// Wraps [`ParseIntError`](std::num::ParseIntError).
    ParseIntError(std::num::ParseIntError),
    /// Wraps [`image::ImageError`].
    ImageError(image::ImageError),
    /// Wraps [`serde_json::Error`].
    JsonDecodeError(Backtrace, serde_json::Error, Option<PathBuf>),
    /// Wraps [`serde_json::Error`].
    JsonEncodeError(Backtrace, serde_json::Error, Option<PathBuf>),
    /// Raised when the CRC field of the requested archive is unequal
    /// to the one in its [`Metadata`](crate::meta::Metadata).
    CrcError(u32, u32, i64, i64),
    /// Raised when the CRC field of the requested archive is unequal
    /// to the one in its [`Metadata`](crate::meta::Metadata).
    VersionError(u32, u32, i64, i64),
    /// Raised if a file fails during decompression.
    DecompressionError(String),
    /// Raised if the index cannot be found, usually if the cache is missing or malformed.
    CacheNotFoundError(std::io::Error, PathBuf),
    /// Raised if an [`Archive`](crate::arc::Archive) is not in the [`CacheIndex`](crate::index::CacheIndex).
    ArchiveNotFoundError(u32, u32),
    /// Raised if a file is not in an [`Archive`](crate::arc::Archive).
    FileNotFoundError(u32, u32, u32),
}

#[cfg(feature = "rs3")]
impl From<sqlite::Error> for CacheError {
    fn from(cause: sqlite::Error) -> Self {
        Self::SqliteError(cause)
    }
}

impl From<std::io::Error> for CacheError {
    fn from(cause: std::io::Error) -> Self {
        Self::IoError(cause)
    }
}

impl From<std::num::ParseIntError> for CacheError {
    fn from(cause: std::num::ParseIntError) -> Self {
        Self::ParseIntError(cause)
    }
}

impl From<image::ImageError> for CacheError {
    fn from(cause: image::ImageError) -> Self {
        Self::ImageError(cause)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(cause: serde_json::Error) -> Self {
        if cause.is_io() {
            Self::JsonDecodeError(Backtrace::force_capture(), cause, None)
        } else {
            Self::JsonEncodeError(Backtrace::force_capture(), cause, None)
        }
    }
}

impl From<DecodeError> for CacheError {
    fn from(cause: DecodeError) -> Self {
        Self::DecodeError(cause)
    }
}

impl From<&CacheError> for CacheError {
    fn from(error: &CacheError) -> Self {
        // gross workaround for CacheError not being copy/clone-able
        match *error {
            Self::FileNotFoundError(index, archive, file) => Self::FileNotFoundError(index, archive, file),
            _ => unimplemented!("Error can't be cloned"),
        }
    }
}

use std::fmt::{Debug, Display, Formatter};

impl Debug for CacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for CacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use std::error::Error;

        writeln!(f, "An error occurred while interpreting the cache.")?;
        writeln!(f)?;

        // Do some special formatting for the first source error
        match self {
            Self::JsonDecodeError(_, source, Some(file)) => writeln!(
                f,
                "Caused by `serde_json::Error`: {} while deserializing {}",
                source,
                file.to_string_lossy()
            )?,
            Self::CacheNotFoundError(e, file) => writeln!(
                f,
                "Encountered Error: \x1B[91m{:?}\x1B[0m \n while looking for file \x1B[93m{:?}\x1B[0m.\n",
                e,
                file.absolutize().unwrap_or(Cow::Borrowed(file))
            )?,
            Self::CrcError(index_id, archive_id, crc1, crc2) => {
                writeln!(f, "Index {} Archive {}: Crc does not match: {} !=  {}", index_id, archive_id, crc1, crc2)?
            }
            Self::VersionError(index_id, archive_id, v1, v2) => {
                writeln!(f, "Index {} Archive {}: Version does not match: {} !=  {}", index_id, archive_id, v1, v2)?
            }
            Self::ArchiveNotFoundError(5, archive) => writeln!(f, "Index 5 does not contain mapsquare ({}, {})", archive & 0x7F, archive >> 7)?,
            Self::ArchiveNotFoundError(index, archive) => writeln!(f, "Index {} does not contain archive {}", index, archive)?,
            Self::FileNotFoundError(index, archive, file) => writeln!(f, "\nIndex {}, Archive {} does not contain file {}", index, archive, file)?,
            _ => {
                if let Some(source) = self.source() {
                    writeln!(f, "Caused by: {}", source)?;
                }
            }
        }

        // Display deeper source errors, if any.
        for s in <dyn Error>::chain(self).skip(2) {
            writeln!(f, "Caused by: {}", s)?;
        }

        writeln!(f)?;

        if let Some(trace) = self.backtrace() {
            writeln!(f, "The following backtrace was captured:")?;
            writeln!(f, "{}", trace)?;
        }

        Ok(())
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "rs3")]
            Self::SqliteError(ref e) => Some(e),
            Self::DecodeError(ref e) => Some(e),
            Self::IoError(ref e) => Some(e),
            Self::JsonDecodeError(_, ref e, _) => Some(e),
            Self::ImageError(ref e) => Some(e),
            Self::ParseIntError(e) => Some(e),
            _ => None,
        }
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        match self {
            Self::JsonDecodeError(trace, _, _) => Some(trace),
            _ => None,
        }
    }
}

#[cfg(feature = "pyo3")]
mod py_error_impl {
    use pyo3::{exceptions::PyRuntimeError, PyErr};

    use super::CacheError;

    impl From<CacheError> for PyErr {
        fn from(err: CacheError) -> PyErr {
            PyRuntimeError::new_err(err.to_string())
        }
    }

    impl From<&CacheError> for PyErr {
        fn from(err: &CacheError) -> PyErr {
            PyRuntimeError::new_err(err.to_string())
        }
    }
}
