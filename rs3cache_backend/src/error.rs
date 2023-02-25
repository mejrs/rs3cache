use std::{
    backtrace::{Backtrace, BacktraceStatus},
    io,
    panic::{Location, PanicInfo},
    path::{Path, PathBuf},
    sync::Arc,
};

use console::style;
use error::With;

use crate::{
    buf::ReadError,
    decoder::DecodeError,
    index::{CachePath, IntegrityError},
};
pub type CacheResult<T> = Result<T, CacheError>;

/// An error type for things that can go wrong when reading from the cache.
#[derive(error::Error)]
#[top_level]
pub enum CacheError {
    #[error = "Something went wrong when parsing the cache"]
    Decode {
        #[source]
        source: DecodeError,
    },
    #[error = "{source}"]
    Io {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },
    #[error = "{source}"]
    JsonEncode {
        #[source]
        source: serde_json::Error,
        file: Option<PathBuf>,
    },
    #[error = "{msg}"]
    Decompression { msg: String },
    #[error = "Something went wrong when parsing the cache"]
    Read {
        #[source]
        source: ReadError,
    },
    #[error = "Something went wrong when accessing the cache"]
    Integrity {
        #[source]
        source: IntegrityError,
    },
    #[error = "xtea for mapsquare({i}, {j}) is not available"]
    Xtea { i: u8, j: u8 },
    #[error = "could not find xteas at {path:?}"]
    XteaLoad {
        #[source]
        source: serde_json::Error,
        path: PathBuf,
    },
}

#[cfg(feature = "pyo3")]
pub mod py_error_impl {
    use pyo3::{
        exceptions::{PyException, PyRuntimeError},
        PyErr,
    };

    use super::*;

    pyo3::create_exception!(cache, CacheNotFoundError, PyException, "Raised if the cache cannot be found");
    pyo3::create_exception!(cache, ArchiveNotFoundError, PyException, "Raised if an archive is missing");
    pyo3::create_exception!(cache, FileMissingError, PyException, "Raised if a file in an archive is missing");
    #[cfg(feature = "dat2")]
    pyo3::create_exception!(cache, XteaError, PyException, "Raised if something related to an xtea went wrong");

    impl From<CacheError> for PyErr {
        fn from(err: CacheError) -> PyErr {
            PyErr::from(&err)
        }
    }

    impl From<&CacheError> for PyErr {
        fn from(err: &CacheError) -> PyErr {
            match err {
                CacheError::Integrity {
                    source: IntegrityError::CannotOpen { .. },
                } => CacheNotFoundError::new_err(err.to_string()),
                CacheError::Integrity {
                    source: IntegrityError::ArchiveMissing { .. },
                } => ArchiveNotFoundError::new_err(err.to_string()),
                #[cfg(feature = "dat2")]
                CacheError::Integrity {
                    source: IntegrityError::ArchiveMissingNamed { .. },
                } => ArchiveNotFoundError::new_err(err.to_string()),
                CacheError::Integrity {
                    source: IntegrityError::FileMissing { .. },
                } => FileMissingError::new_err(err.to_string()),
                #[cfg(feature = "dat2")]
                CacheError::Xtea { .. } | CacheError::XteaLoad { .. } => XteaError::new_err(err.to_string()),
                _ => PyRuntimeError::new_err(err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let e = CacheError::Xtea { i: 42, j: 73 };
        let s = e.to_string();
        assert_eq!(s, "xtea for mapsquare(42, 73) is not available\n");
    }
}
