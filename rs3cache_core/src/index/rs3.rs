use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env::{self, VarError},
    fs::{self, File},
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use bytes::{Buf, Bytes};
use fstrings::{f, format_args_f};
use itertools::iproduct;
use path_macro::path;

use crate::{
    arc::Archive,
    buf::BufExtra,
    decoder,
    error::{CacheError, CacheResult},
    index::{CacheIndex, IndexState, Initial},
    indextype::IndexType,
    meta::{IndexMetadata, Metadata},
};

impl<S> CacheIndex<S>
where
    S: IndexState,
{
    /// Loads the [`Metadata`] of `self`.
    #[allow(unused_variables)]
    #[cfg(not(feature = "mockdata"))]
    fn get_raw_metadata(index_id: u32, connection: &sqlite::Connection) -> CacheResult<Bytes> {
        let encoded_data = {
            let query = "SELECT DATA FROM cache_index";
            let mut statement = connection.prepare(query)?;
            statement.next()?;
            statement.read::<Vec<u8>>(0)?
        };

        #[cfg(feature = "save_mockdata")]
        {
            std::fs::create_dir_all("test_data/mockdata".to_string()).unwrap();
            let filename = format!("test_data/mockdata/cache_index_{}", index_id);
            let mut file = File::create(&filename).unwrap();
            file.write_all(&encoded_data).unwrap();
        }
        Ok(decoder::decompress(encoded_data, None)?)
    }

    /// Grabs mock data from disk.
    #[cfg(feature = "mockdata")]
    fn get_raw_metadata(index_id: u32) -> CacheResult<Bytes> {
        let filename = format!("test_data/mockdata/cache_index_{}", index_id);

        let mut file = File::open(&filename)?;
        let mut encoded_data = Vec::new();
        file.read_to_end(&mut encoded_data)?;

        Ok(decoder::decompress(encoded_data, None)?)
    }

    /// Executes a sql command to retrieve an archive from the cache.
    #[cfg(not(feature = "mockdata"))]
    pub fn get_file(&self, metadata: &Metadata) -> CacheResult<Bytes> {
        let encoded_data = {
            let query = format!("SELECT DATA, CRC, VERSION FROM cache WHERE KEY={}", metadata.archive_id());

            let mut statement = self.connection.prepare(&query)?;
            statement.next()?;
            let crc = statement.read::<i64>(1)?;
            let version = statement.read::<i64>(2)?;

            // wut
            let crc_offset = match self.index_id() {
                IndexType::SPRITES => 2_i64,
                IndexType::MODELSRT7 => 2_i64,
                _ => 1_i64,
            };

            if crc == 0 && version == 0 {
                Err(CacheError::ArchiveNotFoundError(metadata.index_id(), metadata.archive_id()))
            } else if metadata.crc() as i64 + crc_offset != crc {
                Err(CacheError::CrcError(
                    metadata.index_id(),
                    metadata.archive_id(),
                    metadata.crc() as i64 + crc_offset,
                    crc,
                ))
            } else if metadata.version() as i64 != version {
                Err(CacheError::VersionError(
                    metadata.index_id(),
                    metadata.archive_id(),
                    metadata.version() as i64,
                    version,
                ))
            } else {
                Ok(statement.read::<Vec<u8>>(0)?)
            }
        }?;

        #[cfg(feature = "save_mockdata")]
        {
            std::fs::create_dir_all("test_data/mockdata".to_string()).unwrap();
            let filename = format!("test_data/mockdata/index_{}_archive_{}", self.index_id(), metadata.archive_id());
            let mut file = File::create(&filename).unwrap();
            file.write_all(&encoded_data).unwrap()
        }
        Ok(decoder::decompress(encoded_data, metadata.size())?)
    }
    /// Grabs mock data from disk.
    #[cfg(feature = "mockdata")]
    pub fn get_file(&self, metadata: &Metadata) -> CacheResult<Bytes> {
        let filename = format!("test_data/mockdata/index_{}_archive_{}", self.index_id(), metadata.archive_id());
        let encoded_data = fs::read(&filename)?;
        Ok(decoder::decompress(encoded_data, metadata.size())?)
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
    #[cfg(not(any(feature = "mockdata", feature = "save_mockdata")))]
    pub fn assert_coherence(&self) -> CacheResult<()> {
        for (archive_id, metadata) in self.metadatas().iter() {
            let query = format!("SELECT CRC, VERSION FROM cache WHERE KEY={}", archive_id);

            let mut statement = self.connection.prepare(&query)?;
            statement.next()?;
            let crc = statement.read::<i64>(0)?;
            let version = statement.read::<i64>(1)?;

            // wut
            let crc_offset = match self.index_id() {
                IndexType::SPRITES => 2_i64,
                IndexType::MODELSRT7 => 2_i64,
                _ => 1_i64,
            };
            if crc == 0 && version == 0 {
                return Err(CacheError::ArchiveNotFoundError(metadata.index_id(), metadata.archive_id()));
            } else if metadata.crc() as i64 + crc_offset != crc {
                return Err(CacheError::CrcError(
                    metadata.index_id(),
                    metadata.archive_id(),
                    metadata.crc() as i64 + crc_offset,
                    crc,
                ));
            } else if metadata.version() as i64 != version {
                return Err(CacheError::VersionError(
                    metadata.index_id(),
                    metadata.archive_id(),
                    metadata.version() as i64,
                    version,
                ));
            }
        }
        Ok(())
    }
}

impl CacheIndex<Initial> {
    /// Constructor for [`CacheIndex`].
    ///
    /// # Errors
    ///
    /// Raises [`CacheNotFoundError`](CacheError::CacheNotFoundError) if the cache database cannot be found.
    #[cfg(not(feature = "mockdata"))]
    pub fn new(index_id: u32, folder: impl AsRef<Path>) -> CacheResult<CacheIndex<Initial>> {
        let file = path!(folder / f!("js5-{index_id}.jcache"));

        // check if database exists (without creating blank sqlite databases)
        match fs::metadata(&file) {
            Ok(_) => {
                let connection = sqlite::open(file)?;
                let raw_metadata: Bytes = Self::get_raw_metadata(index_id, &connection)?;
                let metadatas = IndexMetadata::deserialize(index_id, raw_metadata)?;

                Ok(Self {
                    index_id,
                    metadatas,
                    connection,
                    path: folder.as_ref().to_path_buf(),
                    state: Initial {},
                })
            }
            //_ => panic!(),
            Err(e) => Err(CacheError::CacheNotFoundError(e, file)),
        }
    }

    /// Mock constructor for `CacheIndex`.
    #[cfg(feature = "mockdata")]
    pub fn new(index_id: u32, folder: impl AsRef<Path>) -> CacheResult<Self> {
        let raw_metadata = Self::get_raw_metadata(index_id)?;
        let metadatas = IndexMetadata::deserialize(index_id, raw_metadata)?;

        Ok(CacheIndex {
            index_id,
            path: folder.as_ref().to_path_buf(),
            connection: PhantomData,
            metadatas,
            state: Initial {},
        })
    }
}

/// Asserts whether all indices' metadata match their contents.
/// Indices 14, 40, 54, 55 are not necessarily complete.
///
/// Exposed as `--assert coherence`.
///
/// # Panics
/// Panics if compiled with feature `mockdata`.
#[cfg(all(feature = "rs3", not(any(feature = "mockdata", feature = "save_mockdata"))))]
pub fn assert_coherence(folder: impl AsRef<Path>) -> CacheResult<()> {
    for index_id in IndexType::iterator() {
        match CacheIndex::new(index_id, &folder)?.assert_coherence() {
            Ok(_) => println!("Index {} is coherent!", index_id),
            Err(e) => println!("Index {} is not coherent: {} and possibly others.", index_id, e),
        }
    }
    Ok(())
}
