use std::{
    backtrace::{Backtrace, BacktraceStatus},
    io,
    panic::{Location, PanicInfo},
    path::{Path, PathBuf},
    sync::Arc,
};

use console::style;

use crate::{
    buf::ReadError,
    decoder::DecodeError,
    index::{CachePath, IntegrityError},
};

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

pub trait Context<T, Src>: Sized {
    fn context<W: With<Src, Dst>, Dst>(self, ctx: W) -> Result<T, Dst>;
}

pub trait With<Src, Dst> {
    fn bind(self, src: Src) -> Dst;
}

impl<T, Src> Context<T, Src> for Result<T, Src> {
    #[track_caller]
    fn context<W: With<Src, Dst>, Dst>(self, ctx: W) -> Result<T, Dst> {
        match self {
            Ok(v) => Ok(v),
            Err(cause) => Err(ctx.bind(cause)),
        }
    }
}

impl With<io::Error, CacheError> for PathBuf {
    #[track_caller]
    fn bind(self, e: io::Error) -> CacheError {
        CacheError {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::IoError(e, self),
                backtrace: Backtrace::capture(),
                location: Location::caller(),
            }),
        }
    }
}

impl With<io::Error, CacheError> for &PathBuf {
    #[track_caller]
    fn bind(self, e: io::Error) -> CacheError {
        self.to_path_buf().bind(e)
    }
}

impl With<io::Error, CacheError> for &Path {
    #[track_caller]
    fn bind(self, e: io::Error) -> CacheError {
        self.to_path_buf().bind(e)
    }
}

impl CacheError {
    pub fn kind(&self) -> &CacheErrorKind {
        &self.inner.kind
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
    DecodeError(DecodeError),
    /// Wraps [`io.error`](std::io::Error).
    IoError(std::io::Error, PathBuf),
    /// Wraps [`serde_json::Error`].
    JsonEncodeError(serde_json::Error, Option<PathBuf>),
    /// Raised if a file fails during decompression.
    DecompressionError(String),
    /// Raised if reading from a buffer fails
    ReadError(ReadError),
    /// Raised from a database
    IntegrityError(IntegrityError),
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
impl From<IntegrityError> for CacheError {
    #[track_caller]
    fn from(cause: IntegrityError) -> Self {
        Self {
            inner: Arc::new(Inner {
                kind: CacheErrorKind::IntegrityError(cause),
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

        writeln!(
            f,
            "An error occurred while interpreting the cache, at {}",
            style(self.inner.location).yellow()
        )?;
        writeln!(f)?;

        // Do some special formatting for the first source error
        match self.kind() {
            #[cfg(feature = "dat2")]
            CacheErrorKind::XteaLoadError(source, file) => writeln!(f, "Caused by `serde_json::Error`: {source} while deserializing {file:?}")?,
            _ => {
                if let Some(source) = self.source() {
                    write!(f, "Caused by: {source}")?;
                }
            }
        }

        // Display deeper source errors, if any.
        for s in <dyn Error>::sources(self).skip(2) {
            writeln!(f, "Caused by: {s}")?;
        }

        writeln!(f)?;

        if let Some(trace) = <dyn Error>::request_ref::<Backtrace>(self) {
            match trace.status() {
                BacktraceStatus::Disabled => writeln!(
                    f,
                    "No backtrace was captured. Set `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE` to capture a backtrace."
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
pub const STRUCTURE: &str = if cfg!(feature = "sqlite") {
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
                CacheErrorKind::IntegrityError(IntegrityError::CannotOpen { .. }) => CacheNotFoundError::new_err(err.to_string()),
                CacheErrorKind::IntegrityError(IntegrityError::Missing { .. }) => ArchiveNotFoundError::new_err(err.to_string()),
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
