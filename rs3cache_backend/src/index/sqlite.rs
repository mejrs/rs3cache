use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env::{self, VarError},
    fmt::{Display, Formatter},
    fs::{self, File},
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::{Buf, Bytes};
use console::style;
use itertools::iproduct;
use path_macro::path;
use rusqlite::{Connection, OpenFlags};

use crate::{
    arc::Archive,
    buf::BufExtra,
    decoder,
    error::{CacheError, CacheResult, Context, With, STRUCTURE},
    index::{CacheIndex, CachePath, IndexState, Initial},
    meta::{IndexMetadata, Metadata},
};

impl CacheIndex<Initial> {
    /// Constructor for [`CacheIndex`].
    ///
    /// # Errors
    ///
    /// Raises [`CacheNotFoundError`](CacheError::CacheNotFoundError) if the cache database cannot be found.
    pub fn new(index_id: u32, path: Arc<CachePath>) -> CacheResult<CacheIndex<Initial>> {
        let file = path!(*path / format!("js5-{index_id}.jcache"));

        let connection = Connection::open_with_flags(&file, OpenFlags::SQLITE_OPEN_READ_ONLY).context(CannotOpen { file, input: path.clone() })?;
        let data = connection
            .query_row("SELECT DATA FROM cache_index", [], |row| row.get(0))
            .context(Other)?;
        let raw_metadata = decoder::decompress(data)?;
        let metadatas = IndexMetadata::deserialize(index_id, raw_metadata)?;

        Ok(Self {
            index_id,
            metadatas,
            connection,
            path,
            state: Initial {},
        })
    }
}

impl<S> CacheIndex<S>
where
    S: IndexState,
{
    /// Executes a sql command to retrieve an archive from the cache.
    pub fn get_file(&self, metadata: &Metadata) -> CacheResult<Bytes> {
        let (data, crc, version) = self
            .connection
            .query_row("SELECT DATA, CRC, VERSION FROM cache WHERE KEY=?", [metadata.archive_id()], |row| try {
                (row.get("DATA")?, row.get("CRC")?, row.get("VERSION")?)
            })
            .context(Missing { metadata: metadata.clone() })?;

        // wut
        let crc_offset = match self.index_id() {
            8 => 2_i64,
            47 => 2_i64,
            _ => 1_i64,
        };

        if crc == 0 && version == 0 {
            Err(IntegrityError::Blank { metadata: metadata.clone() }.into())
        } else if metadata.crc() as i64 + crc_offset != crc {
            Err(IntegrityError::Crc {
                crc,
                metadata: metadata.clone(),
            }
            .into())
        } else if metadata.version() as i64 != version {
            Err(IntegrityError::Version {
                version,
                metadata: metadata.clone(),
            }
            .into())
        } else {
            Ok(decoder::decompress(data)?)
        }
    }

    /// Assert whether the cache held by `self` is in a coherent state.
    ///
    /// # Errors
    ///
    /// May raise [`CrcError`](CacheError::CrcError), [`VersionError`](CacheError::VersionError) or [`ArchiveNotFoundError`](CacheError::ArchiveNotFoundError)
    /// if the cache is not in a logical state.
    ///
    /// # Notes
    /// Indices `VORBIS`, `AUDIOSTREAMS`, `TEXTURES_PNG_MIPPED` and `TEXTURES_ETC` tend to never complete.
    /// For these, simply ignore [`ArchiveNotFoundError`](CacheError::ArchiveNotFoundError).
    pub fn assert_coherence(&self) -> Result<(), IntegrityError> {
        for (_, metadata) in self.metadatas().iter() {
            let (_, crc, version) = self
                .connection
                .query_row("SELECT DATA, CRC, VERSION FROM cache WHERE KEY=?", [metadata.archive_id()], |row| try {
                    (row.get::<_, Vec<u8>>("DATA")?, row.get("CRC")?, row.get("VERSION")?)
                })
                .context(Missing { metadata: metadata.clone() })?;

            // wut
            let crc_offset = match self.index_id() {
                8 => 2_i64,
                47 => 2_i64,
                _ => 1_i64,
            };
            if crc == 0 && version == 0 {
                return Err(IntegrityError::Blank { metadata: metadata.clone() });
            } else if metadata.crc() as i64 + crc_offset != crc {
                return Err(IntegrityError::Crc {
                    crc,
                    metadata: metadata.clone(),
                });
            } else if metadata.version() as i64 != version {
                return Err(IntegrityError::Version {
                    version,
                    metadata: metadata.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum IntegrityError {
    CannotOpen {
        source: rusqlite::Error,
        file: PathBuf,
        input: Arc<CachePath>,
    },
    Missing {
        metadata: Metadata,
    },
    Crc {
        crc: i64,
        metadata: Metadata,
    },
    Version {
        version: i64,
        metadata: Metadata,
    },
    Blank {
        metadata: Metadata,
    },
    Corrupted {
        source: rusqlite::Error,
        metadata: Metadata,
    },
    Other {
        source: rusqlite::Error,
    },
}

pub struct CannotOpen {
    pub file: PathBuf,
    pub input: Arc<CachePath>,
}
pub struct Missing {
    pub metadata: Metadata,
}
pub struct Crc {
    pub crc: i64,
    pub metadata: Metadata,
}
pub struct Version {
    pub version: i64,
    pub metadata: Metadata,
}
pub struct Blank {
    pub metadata: Metadata,
}
pub struct Corrupted {
    pub metadata: Metadata,
}

struct Other;

impl Display for IntegrityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            IntegrityError::CannotOpen { source, file, input } => {
                let path = path_absolutize::Absolutize::absolutize(file).unwrap_or(std::borrow::Cow::Borrowed(file));
                writeln!(
                    f,
                    "cannot open cache: encountered {} while looking for file {}",
                    style(source).yellow(),
                    style(path.display()).yellow()
                )?;
                match &**input {
                    CachePath::Given(path) => writeln!(
                        f,
                        "note: looking in this location because the path {} was given as an argument",
                        style(path.display()).yellow()
                    )?,
                    CachePath::Env(path) => writeln!(
                        f,
                        "note: looking in this location because the path {} was retrieved from an environment variable",
                        style(path.display()).yellow()
                    )?,
                    CachePath::Omitted => writeln!(f, "note: looking in the current directory because no path was given")?,
                }
                let path = (**input).as_ref();
                let path = path_absolutize::Absolutize::absolutize(path).unwrap_or(std::borrow::Cow::Borrowed(path));
                let path = path.to_string_lossy();

                writeln!(f, "note: expecting the following folder structure:")?;
                writeln!(f, "    {path}{STRUCTURE}")?;
            }
            IntegrityError::Crc {
                crc: crc1,
                metadata:
                    Metadata {
                        index_id,
                        archive_id,
                        crc: crc2,
                        ..
                    },
            } => write!(f, "Index {index_id} Archive {archive_id}: Crc does not match: {crc1} !=  {crc2}")?,
            IntegrityError::Version {
                version: v1,
                metadata:
                    Metadata {
                        index_id,
                        archive_id,
                        version: v2,
                        ..
                    },
            } => write!(f, "Index {index_id} Archive {archive_id}: Version does not match: {v1} !=  {v2}")?,
            IntegrityError::Missing {
                metadata: Metadata { index_id: 5, archive_id, .. },
            } => write!(f, "Index 5 does not contain mapsquare ({}, {})", archive_id & 0x7F, archive_id >> 7)?,
            IntegrityError::Missing {
                metadata: Metadata { index_id, archive_id, .. },
            } => writeln!(f, "Index {index_id} does not contain archive {archive_id}")?,
            IntegrityError::Blank {
                metadata: Metadata { index_id, archive_id, .. },
            } => writeln!(f, "Index {index_id}'s archive {archive_id} is blank")?,
            IntegrityError::Corrupted { source, metadata } => writeln!(f, "Error retrieving {metadata}: {source}")?,
            IntegrityError::Other { source } => Display::fmt(source, f)?,
        }

        Ok(())
    }
}

impl With<rusqlite::Error, IntegrityError> for Other {
    fn bind(self, source: rusqlite::Error) -> IntegrityError {
        let Other {} = self;
        IntegrityError::Other { source }
    }
}

impl With<rusqlite::Error, IntegrityError> for Corrupted {
    fn bind(self, source: rusqlite::Error) -> IntegrityError {
        let Corrupted { metadata } = self;
        IntegrityError::Corrupted { source, metadata }
    }
}

impl With<rusqlite::Error, IntegrityError> for Missing {
    fn bind(self, source: rusqlite::Error) -> IntegrityError {
        let Missing { metadata } = self;
        IntegrityError::Corrupted { source, metadata }
    }
}

impl With<rusqlite::Error, IntegrityError> for CannotOpen {
    fn bind(self, source: rusqlite::Error) -> IntegrityError {
        use rusqlite::Error::*;

        let CannotOpen { file, input } = self;
        match source {
            SqliteFailure(
                libsqlite3_sys::Error {
                    code: libsqlite3_sys::ErrorCode::CannotOpen,
                    ..
                },
                ..,
            ) => IntegrityError::CannotOpen { source, file, input },
            source => IntegrityError::Other { source },
        }
    }
}

/// Asserts whether all indices' metadata match their contents.
/// Indices 14, 40, 54, 55 are not necessarily complete.
///
/// Exposed as `--assert coherence`.
///
/// # Panics
///
/// Panics if compiled with feature `mockdata`.
#[cfg(not(feature = "mockdata"))]
pub fn assert_coherence(folder: Arc<CachePath>) -> CacheResult<()> {
    for index_id in 0..70 {
        if fs::metadata(path!(&*folder / format!("js5-{index_id}.jcache"))).is_ok() {
            match CacheIndex::new(index_id, folder.clone())?.assert_coherence() {
                Ok(_) => println!("Index {index_id} is coherent!"),
                Err(e) => println!("Index {index_id} is not coherent: {e} and possibly others."),
            }
        }
    }
    Ok(())
}
