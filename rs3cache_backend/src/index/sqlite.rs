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

use ::error::{Context, With};
use bytes::{Buf, Bytes};
use console::style;
use itertools::iproduct;
use path_macro::path;
use rusqlite::{Connection, OpenFlags};

use crate::{
    arc::Archive,
    buf::BufExtra,
    decoder,
    error::{self, CacheError, CacheResult, CannotOpen},
    index::*,
    meta::{IndexMetadata, Metadata},
};

impl CacheIndex<Initial> {
    /// Constructor for [`CacheIndex`].
    ///
    /// # Errors
    ///
    /// Raises [`CacheNotFoundError`](CacheError::CacheNotFoundError) if the cache database cannot be found.
    pub fn new(index_id: u32, input: Arc<CachePath>) -> CacheResult<CacheIndex<Initial>> {
        let file = path!(*input / format!("js5-{index_id}.jcache"));

        let connection = Connection::open_with_flags(&file, OpenFlags::SQLITE_OPEN_READ_ONLY).with_context(|| CannotOpen {
            file: file.clone(),
            input: input.clone(),
        })?;
        let data = connection
            .query_row("SELECT DATA FROM cache_index", [], |row| row.get(0))
            .context(Other)
            .context(error::Integrity)?;
        let raw_metadata = decoder::decompress(data).context(error::Decode)?;
        let metadatas = IndexMetadata::deserialize(index_id, raw_metadata).context(error::Read)?;

        Ok(Self {
            index_id,
            metadatas,
            connection,
            input,
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
            .context(ArchiveMissing {
                index_id: metadata.index_id(),
                archive_id: metadata.archive_id(),
            })
            .context(error::Integrity)?;

        // wut
        let crc_offset = match self.index_id() {
            8 => 2_i64,
            47 => 2_i64,
            _ => 1_i64,
        };

        if crc == 0 && version == 0 {
            Err(IntegrityError::Blank { metadata: metadata.clone() }).context(error::Integrity)
        } else if metadata.crc() as i64 + crc_offset != crc {
            Err(IntegrityError::Crc {
                crc,
                metadata: metadata.clone(),
            })
            .context(error::Integrity)
        } else if metadata.version() as i64 != version {
            Err(IntegrityError::Version {
                version,
                metadata: metadata.clone(),
            })
            .context(error::Integrity)
        } else {
            decoder::decompress(data).context(error::Decode)
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
                .context(ArchiveMissing {
                    index_id: metadata.index_id(),
                    archive_id: metadata.archive_id(),
                })?;

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
