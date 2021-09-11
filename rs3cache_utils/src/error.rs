use std::{backtrace::Backtrace, borrow::Cow, path::PathBuf};

use path_absolutize::*;

/*
#[cfg(feature = "rs3")]
use crate::cache::indextype::{IndexType, MapFileType};
*/

/// Result wrapper for [`CacheError`].
pub type CacheResult<T> = Result<T, CacheError>;

/// An error type for things that can go wrong when reading from the cache.
pub enum CacheError {
    /// Wraps [`sqlite::Error`](https://docs.rs/sqlite/0.25.3/sqlite/struct.Error.html).
    #[cfg(feature = "rs3")]
    SqliteError(sqlite::Error, Backtrace),
    /// Wraps [`io.error`](std::io::Error).
    IoError(std::io::Error, Backtrace),
    /// Wraps [`ParseIntError`](std::num::ParseIntError).
    ParseIntError(std::num::ParseIntError, Backtrace),
    /// Wraps [`ImageError`](https://docs.rs/image/0.23.14/image/error/enum.ImageError.html).
    ImageError(image::ImageError, Backtrace),
    /// Wraps [`serde_json::Error`](https://docs.serde.rs/serde_json/struct.Error.html).
    SerdeError(serde_json::Error, Backtrace),
    /// Wraps [`bzip2::Error`](https://docs.rs/bzip2/0.4.2/bzip2/enum.Error.html).
    BZip2Error(bzip2::Error, Backtrace),
    /// Raised when the CRC field of the requested archive is unequal
    /// to the one in its [`Metadata`](crate::cache::meta::Metadata).
    CrcError(u32, u32, i64, i64),
    /// Raised when the CRC field of the requested archive is unequal
    /// to the one in its [`Metadata`](crate::cache::meta::Metadata).
    VersionError(u32, u32, i64, i64),
    /// Wrapper for [`NoneError`](std::option::NoneError).
    NoneError(Backtrace),
    /// Raised if a file fails during decompression.
    DecompressionError(String, Backtrace),
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
        Self::SqliteError(cause, Backtrace::capture())
    }
}

impl From<bzip2::Error> for CacheError {
    fn from(cause: bzip2::Error) -> Self {
        Self::BZip2Error(cause, Backtrace::capture())
    }
}

impl From<std::io::Error> for CacheError {
    fn from(cause: std::io::Error) -> Self {
        Self::IoError(cause, Backtrace::capture())
    }
}

impl From<std::num::ParseIntError> for CacheError {
    fn from(cause: std::num::ParseIntError) -> Self {
        Self::ParseIntError(cause, Backtrace::capture())
    }
}

impl From<image::ImageError> for CacheError {
    fn from(cause: image::ImageError) -> Self {
        Self::ImageError(cause, Backtrace::capture())
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(cause: serde_json::Error) -> Self {
        Self::SerdeError(cause, Backtrace::capture())
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

impl Display for CacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl Debug for CacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "rs3")]
            Self::SqliteError(ref err, trace) => write!(f, "{}\n\n{:#?}", err, trace),
            Self::IoError(ref err, trace) => write!(f, "{}\n\n{:#?}", err, trace),
            Self::SerdeError(ref err, trace) => write!(f, "{}\n\n{:#?}", err, trace),
            Self::ImageError(ref err, trace) => write!(f, "{}\n\n{:#?}", err, trace),
            Self::ParseIntError(msg, trace) => write!(f, "{}\n\n{:#?}", msg, trace),
            Self::NoneError(trace) => write!(f, "An Option contained a None value.\n{:#?}", trace),

            Self::BZip2Error(bzip2::Error::Sequence, trace) => write!(
                f,
                "The sequence of operations called on a decompression/compression buffer were invalid. {}",
                trace
            ),
            Self::BZip2Error(bzip2::Error::Data, trace) => {
                write!(f, "The data being decompressed was invalid, or it was not a valid bz2 buffer. {}", trace)
            }
            Self::BZip2Error(bzip2::Error::DataMagic, trace) => write!(f, "The magic bz2 header wasn't present when decompressing.\n {}", trace),
            Self::BZip2Error(bzip2::Error::Param, trace) => write!(f, "The parameters to the bzip2 deserializer were invalid.\n {}", trace),

            Self::CrcError(index_id, archive_id, crc1, crc2) => {
                write!(f, "Index {} Archive {}: Crc does not match: {} !=  {}", index_id, archive_id, crc1, crc2)
            }
            Self::VersionError(index_id, archive_id, v1, v2) => {
                write!(f, "Index {} Archive {}: Version does not match: {} !=  {}", index_id, archive_id, v1, v2)
            }

            Self::DecompressionError(msg, trace) => write!(f, "An error occurred while uncompressing: {}\n\n{:#?}", msg, trace),
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
            /*

            #[cfg(feature = "rs3")]
            Self::FileNotFoundError(IndexType::MAPSV2, archive, MapFileType::LOCATIONS) => {
                write!(f, "Mapsquare ({}, {}) does not contain locations.", archive & 0x7F, archive >> 7)
            }
            #[cfg(feature = "rs3")]
            Self::FileNotFoundError(IndexType::MAPSV2, archive, MapFileType::WATER_LOCATIONS) => {
                write!(f, "Mapsquare ({}, {}) does not contain water locations.", archive & 0x7F, archive >> 7)
            }
            #[cfg(feature = "rs3")]
            Self::FileNotFoundError(IndexType::MAPSV2, archive, MapFileType::TILES) => {
                write!(f, "Mapsquare ({}, {}) does not contain tiles.", archive & 0x7F, archive >> 7)
            }
            #[cfg(feature = "rs3")]
            Self::FileNotFoundError(IndexType::MAPSV2, archive, MapFileType::WATER_TILES) => {
                write!(f, "Mapsquare ({}, {}) does not contain water tiles.", archive & 0x7F, archive >> 7)
            }

            #[cfg(feature = "rs3")]
            Self::FileNotFoundError(IndexType::MAPSV2, archive, file) => {
                write!(f, "Mapsquare ({}, {}) does not contain file {}", archive & 0x3F, archive >> 7, file)
            }
            */
            Self::FileNotFoundError(index, archive, file) => write!(f, "\nIndex {}, Archive {} does not contain file {}", index, archive, file),
        }
    }
}

impl std::error::Error for CacheError {
    fn backtrace(&self) -> Option<&Backtrace> {
        match self {
            #[cfg(feature = "rs3")]
            Self::SqliteError(_, trace) => Some(trace),
            Self::IoError(_, trace) => Some(trace),
            Self::SerdeError(_, trace) => Some(trace),
            Self::ImageError(_, trace) => Some(trace),
            Self::ParseIntError(_, trace) => Some(trace),
            Self::NoneError(trace) => Some(trace),
            Self::BZip2Error(_, trace) => Some(trace),
            _ => None,
        }
    }
}

#[cfg(feature = "pyo3")]
mod py_error_impl {
    use pyo3::{exceptions, PyErr};

    use super::CacheError;

    impl From<CacheError> for PyErr {
        fn from(err: CacheError) -> PyErr {
            match err {
                e @ CacheError::ArchiveNotFoundError(..) => exceptions::PyValueError::new_err(e.to_string()),
                e @ CacheError::CacheNotFoundError(..) => exceptions::PyFileNotFoundError::new_err(e.to_string()),
                e @ CacheError::FileNotFoundError(..) => exceptions::PyFileNotFoundError::new_err(e.to_string()),
                e => exceptions::PyRuntimeError::new_err(e.to_string()),
            }
        }
    }

    impl From<&CacheError> for PyErr {
        fn from(err: &CacheError) -> PyErr {
            match err {
                e @ CacheError::ArchiveNotFoundError(..) => exceptions::PyValueError::new_err(e.to_string()),
                e @ CacheError::CacheNotFoundError(..) => exceptions::PyFileNotFoundError::new_err(e.to_string()),
                e @ CacheError::FileNotFoundError(..) => exceptions::PyFileNotFoundError::new_err(e.to_string()),
                e => exceptions::PyRuntimeError::new_err(e.to_string()),
            }
        }
    }
}
