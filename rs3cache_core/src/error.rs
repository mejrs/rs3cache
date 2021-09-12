use std::{borrow::Cow, path::PathBuf};

use path_absolutize::*;

use crate::decoder::DecodeError;
/// Result wrapper for [`CacheError`].
pub type CacheResult<T> = Result<T, CacheError>;

/// An error type for things that can go wrong when reading from the cache.
#[derive(Debug)]
pub enum CacheError {
    /// Wraps [`sqlite::Error`](https://docs.rs/sqlite/0.25.3/sqlite/struct.Error.html).
    #[cfg(feature = "rs3")]
    SqliteError(sqlite::Error),
    DecodeError(DecodeError),
    /// Wraps [`io.error`](std::io::Error).
    IoError(std::io::Error),
    /// Wraps [`ParseIntError`](std::num::ParseIntError).
    ParseIntError(std::num::ParseIntError),
    /// Wraps [`ImageError`](https://docs.rs/image/0.23.14/image/error/enum.ImageError.html).
    ImageError(image::ImageError),
    /// Wraps [`serde_json::Error`](https://docs.serde.rs/serde_json/struct.Error.html).
    SerdeError(serde_json::Error),
    /// Raised when the CRC field of the requested archive is unequal
    /// to the one in its [`Metadata`](crate::cache::meta::Metadata).
    CrcError(u32, u32, i64, i64),
    /// Raised when the CRC field of the requested archive is unequal
    /// to the one in its [`Metadata`](crate::cache::meta::Metadata).
    VersionError(u32, u32, i64, i64),
    /// Raised if a file fails during decompression.
    DecompressionError(String),
    /// Raised if the index cannot be found, usually if the cache is missing or malformed.
    CacheNotFoundError(std::io::Error, PathBuf),
    /// Raised if an [`Archive`](crate::cache::arc::Archive) is not in the [`CacheIndex`](crate::cache::index::CacheIndex).
    ArchiveNotFoundError(u32, u32),
    /// Raised if a file is not in an [`Archive`](crate::cache::arc::Archive).
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
        Self::SerdeError(cause)
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

use std::fmt::{Display, Formatter};

impl Display for CacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "rs3")]
            Self::SqliteError(ref e) => Display::fmt(&e, f),
            Self::DecodeError(ref e) => Display::fmt(&e, f),
            Self::IoError(ref e) => Display::fmt(&e, f),
            Self::SerdeError(ref e) => Display::fmt(&e, f),
            Self::ImageError(ref e) => Display::fmt(&e, f),
            Self::ParseIntError(e) => Display::fmt(&e, f),
            Self::DecompressionError(e) => Display::fmt(&e, f),
            Self::CrcError(index_id, archive_id, crc1, crc2) => {
                write!(f, "Index {} Archive {}: Crc does not match: {} !=  {}", index_id, archive_id, crc1, crc2)
            }
            Self::VersionError(index_id, archive_id, v1, v2) => {
                write!(f, "Index {} Archive {}: Version does not match: {} !=  {}", index_id, archive_id, v1, v2)
            }

            Self::CacheNotFoundError(e, file) => {
                let msg = format!(
                    "Encountered Error: \x1B[91m{:?}\x1B[0m \n while looking for file \x1B[93m{:?}\x1B[0m.\n",
                    e,
                    file.absolutize().unwrap_or(Cow::Borrowed(file))
                );
                write!(f, "{}", msg)
            }
            Self::ArchiveNotFoundError(5, archive) => write!(f, "Index 5 does not contain mapsquare ({}, {})", archive & 0x7F, archive >> 7),
            Self::ArchiveNotFoundError(index, archive) => write!(f, "Index {} does not contain archive {}", index, archive),
            Self::FileNotFoundError(index, archive, file) => write!(f, "\nIndex {}, Archive {} does not contain file {}", index, archive, file),
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "rs3")]
            Self::SqliteError(ref e) => Some(e),
            Self::DecodeError(ref e) => Some(e),
            Self::IoError(ref e) => Some(e),
            Self::SerdeError(ref e) => Some(e),
            Self::ImageError(ref e) => Some(e),
            Self::ParseIntError(e) => Some(e),
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
