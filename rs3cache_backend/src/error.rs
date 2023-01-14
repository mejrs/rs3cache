use std::{
    backtrace::{Backtrace, BacktraceStatus},
    io,
    panic::{Location, PanicInfo},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{buf::ReadError, decoder::DecodeError, index::CachePath};
/// Result wrapper for [`CacheError`].
pub type CacheResult<T> = Result<T, CacheError>;

/// An error type for things that can go wrong when reading from the cache.
#[derive(Clone)]
pub struct CacheError {
    inner: Arc<Inner>,
}

struct Inner {
    kind: CacheErrorKind,
    backtrace: Backtrace,
    location: &'static Location<'static>,
}

impl CacheError {
    pub fn kind(&self) -> &CacheErrorKind {
        &self.inner.kind
    }

    #[track_caller]
    pub fn archive_missing(index: u32, archive: u32) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::ArchiveNotFoundError(index, archive),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }

    #[track_caller]
    pub fn file_missing(index: u32, archive: u32, file: u32) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::FileMissingError(index, archive, file),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }

    #[track_caller]
    pub fn cache_not_found(io: std::io::Error, path: PathBuf, path2: Arc<CachePath>) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::CacheNotFoundError(io, path, path2),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }

    #[track_caller]
    pub fn crc(index: u32, archive: u32, crc1: i64, crc2: i64) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::CrcError(index, archive, crc1, crc2),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }

    #[track_caller]
    pub fn version(index: u32, archive: u32, v1: i64, v2: i64) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::VersionError(index, archive, v1, v2),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }

    #[track_caller]
    pub fn io(cause: io::Error, path: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::IoError(cause, path),
                backtrace: Backtrace::force_capture(),
                location: Location::caller(),
            }),
        }
    }

    #[cfg(feature = "dat2")]
    pub fn xtea_load_error(cause: serde_json::Error, path: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::XteaLoadError(cause, path),
                backtrace: Backtrace::force_capture(),
                location: Location::caller(),
            }),
        }
    }

    #[cfg(feature = "dat2")]
    pub fn xtea_absent(i: u8, j: u8) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::XteaError { i, j },
                backtrace: Backtrace::force_capture(),
                location: Location::caller(),
            }),
        }
    }
}

pub enum CacheErrorKind {
    /// Wraps [`rusqlite::Error`].
    #[cfg(feature = "sqlite")]
    SqliteError(rusqlite::Error),
    DecodeError(DecodeError),
    /// Wraps [`io.error`](std::io::Error).
    IoError(std::io::Error, PathBuf),
    /// Wraps [`serde_json::Error`].
    JsonEncodeError(serde_json::Error, Option<PathBuf>),
    /// Raised when the CRC field of the requested archive is unequal
    /// to the one in its [`Metadata`](crate::meta::Metadata).
    CrcError(u32, u32, i64, i64),
    /// Raised when the CRC field of the requested archive is unequal
    /// to the one in its [`Metadata`](crate::meta::Metadata).
    VersionError(u32, u32, i64, i64),
    /// Raised if a file fails during decompression.
    DecompressionError(String),
    /// Raised if the index cannot be found, usually if the cache is missing or malformed.
    CacheNotFoundError(std::io::Error, PathBuf, Arc<CachePath>),
    /// Raised if an [`Archive`](crate::arc::Archive) is not in the [`CacheIndex`](crate::index::CacheIndex).
    ArchiveNotFoundError(u32, u32),
    /// Raised if a file is not in an [`Archive`](crate::arc::Archive).
    FileMissingError(u32, u32, u32),
    /// Raised if reading from a buffer fails
    ReadError(ReadError),
    /// ZIf this is raised then likely an xtea is wrong,
    #[cfg(feature = "dat2")]
    XteaError {
        i: u8,
        j: u8,
    },
    /// Wraps [`serde_json::Error`].
    #[cfg(feature = "dat2")]
    XteaLoadError(serde_json::Error, PathBuf),
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for CacheError {
    fn from(cause: rusqlite::Error) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::SqliteError(cause),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }
}

impl From<DecodeError> for CacheError {
    #[track_caller]
    fn from(cause: DecodeError) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::DecodeError(cause),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }
}

impl From<ReadError> for CacheError {
    #[track_caller]
    fn from(cause: ReadError) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::ReadError(cause),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }
}

impl From<&CacheError> for CacheError {
    fn from(error: &CacheError) -> Self {
        error.clone()
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
        match self.kind() {
            #[cfg(feature = "dat2")]
            CacheErrorKind::XteaLoadError(source, file) => writeln!(
                f,
                "Caused by `serde_json::Error`: {} while deserializing {}",
                source,
                file.to_string_lossy()
            )?,
            #[cfg(not(target_arch = "wasm32"))]
            CacheErrorKind::CacheNotFoundError(e, file, input) => {
                let path = path_absolutize::Absolutize::absolutize(file).unwrap_or(std::borrow::Cow::Borrowed(file));
                write!(
                    f,
                    "Encountered Error: \x1B[91m{e:?}\x1B[0m \n while looking for file \x1B[93m{path:?}\x1B[0m.\n",
                )?;
                match &**input {
                    CachePath::Given(path) => writeln!(f, "note: looking in this location because the path {path:?} was given as an argument")?,
                    CachePath::Env(path) => writeln!(
                        f,
                        "note: looking in this location because the path {path:?} was retrieved from an environment variable"
                    )?,
                    CachePath::Omitted => writeln!(f, "note: looking in the current directory because no path was given")?,
                }
                let path = (**input).as_ref();
                let path = path_absolutize::Absolutize::absolutize(path).unwrap_or(std::borrow::Cow::Borrowed(path));
                let path = path.to_string_lossy();

                write!(f, "note: expecting the following folder structure:")?;
                write!(f, "    {path}{STRUCTURE}")?;
            }
            CacheErrorKind::CrcError(index_id, archive_id, crc1, crc2) => {
                write!(f, "Index {index_id} Archive {archive_id}: Crc does not match: {crc1} !=  {crc2}")?
            }
            CacheErrorKind::VersionError(index_id, archive_id, v1, v2) => {
                write!(f, "Index {index_id} Archive {archive_id}: Version does not match: {v1} !=  {v2}")?
            }
            CacheErrorKind::ArchiveNotFoundError(5, archive) => {
                write!(f, "Index 5 does not contain mapsquare ({}, {})", archive & 0x7F, archive >> 7)?
            }
            CacheErrorKind::ArchiveNotFoundError(index, archive) => writeln!(f, "Index {index} does not contain archive {archive}")?,
            CacheErrorKind::FileMissingError(index, archive, file) => write!(f, "\nIndex {index}, Archive {archive} does not contain file {file}")?,
            CacheErrorKind::IoError(io, path) => write!(f, "encountered {io} while handling path {path:?}")?,
            _ => {
                if let Some(source) = self.source() {
                    write!(f, "Caused by: {source}")?;
                }
            }
        }

        writeln!(f, ", at {}", self.inner.location)?;

        // Display deeper source errors, if any.
        for s in <dyn Error>::sources(self).skip(2) {
            writeln!(f, "Caused by: {s}")?;
        }

        writeln!(f)?;

        if let Some(trace) = <dyn Error>::request_ref::<Backtrace>(self) {
            match trace.status() {
                BacktraceStatus::Disabled => writeln!(
                    f,
                    "No backtrace was captured. set `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE` to capture a backtrace."
                )?,
                BacktraceStatus::Captured => writeln!(f, "The following backtrace was captured:\n {trace}")?,
                _ => {}
            }
        }

        Ok(())
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.kind() {
            #[cfg(feature = "sqlite")]
            CacheErrorKind::SqliteError(ref e) => Some(e),
            CacheErrorKind::DecodeError(ref e) => Some(e),
            CacheErrorKind::IoError(ref e, _) => Some(e),
            #[cfg(feature = "dat2")]
            CacheErrorKind::XteaLoadError(ref e, _) => Some(e),
            CacheErrorKind::ReadError(e) => Some(e),
            _ => None,
        }
    }

    fn provide<'a>(&'a self, req: &mut core::any::Demand<'a>) {
        req.provide_ref::<Backtrace>(&self.inner.backtrace);
    }
}

#[cfg(not(target_arch = "wasm32"))]
const STRUCTURE: &str = if cfg!(feature = "sqlite") {
    "/
        js5-1.JCACHE
        js5-2.JCACHE
        ...
        js5-61.JCACHE"
} else if cfg!(feature = "dat2") {
    "/
        cache /
            main_file_cache.dat2
            main_file_cache.idx0
            main_file_cache.idx1
            ...
            main_file_cache.idx21
            main_file_cache.idx255
        xteas.json OR keys.json"
} else if cfg!(feature = "dat") {
    "/
        cache /
            main_file_cache.dat
            main_file_cache.idx0
            main_file_cache.idx1
            main_file_cache.idx2
            main_file_cache.idx3
            main_file_cache.idx4"
} else {
    unimplemented!()
};

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

    impl From<&CacheError> for PyErr {
        fn from(err: &CacheError) -> PyErr {
            match err.kind() {
                CacheErrorKind::CacheNotFoundError(..) => CacheNotFoundError::new_err(err.to_string()),
                CacheErrorKind::ArchiveNotFoundError(..) => ArchiveNotFoundError::new_err(err.to_string()),
                CacheErrorKind::FileMissingError(..) => FileMissingError::new_err(err.to_string()),
                #[cfg(feature = "dat2")]
                CacheErrorKind::XteaError { .. } => XteaError::new_err(err.to_string()),
                _ => PyRuntimeError::new_err(err.to_string()),
            }
        }
    }

    impl From<CacheError> for PyErr {
        fn from(err: CacheError) -> PyErr {
            (&err).into()
        }
    }
}
