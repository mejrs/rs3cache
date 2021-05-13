//! The interface between [rs3cache](crate) and the cache database.

#![allow(unused_imports)] // varies based on mock config flags

#[cfg(all(feature = "mockdata", feature = "save_mockdata"))]
compile_error!("mockdata and save_mockdata are incompatible");

use std::{
    collections::HashSet,
    env::{self, VarError},
    fs::{self, File},
    io::{Read, Write},
    marker::PhantomData,
    ops::RangeInclusive,
    path::Path,
};

use itertools::iproduct;

use crate::{
    cache::{
        arc::{Archive, ArchiveGroup},
        buf::Buffer,
        decoder,
        indextype::IndexType,
        meta::{IndexMetadata, Metadata},
    },
    utils::error::{CacheError, CacheResult},
};

/// System variable for the location of the cache folder.
pub const SYS_VAR: &str = "RUNESCAPE_CACHE_FOLDER";

mod states {
    use std::ops::RangeInclusive;

    /// Initial state of [`CacheIndex`](super::CacheIndex).
    pub struct Initial {}

    pub struct Truncated {
        pub feed: Vec<u32>,
    }

    pub struct Grouped {
        pub dim_i: RangeInclusive<i32>,
        pub dim_j: RangeInclusive<i32>,
    }

    pub struct TruncatedGrouped {
        pub feed: Vec<u32>,
        pub dim_i: RangeInclusive<i32>,
        pub dim_j: RangeInclusive<i32>,
    }

    /// Trait that describes the current index state. Cannot be implemented.
    pub trait IndexState {}
    impl IndexState for Initial {}
    impl IndexState for Truncated {}
    impl IndexState for Grouped {}
    impl IndexState for TruncatedGrouped {}
}

pub use states::Initial;
use states::{Grouped, IndexState, Truncated, TruncatedGrouped};

/// Container of [`Archive`]s.
pub struct CacheIndex<S: IndexState> {
    index_id: u32,
    metadatas: IndexMetadata,
    state: S,

    #[cfg(not(feature = "mockdata"))]
    connection: sqlite::Connection,

    // correctly derive things like !Sync
    #[cfg(feature = "mockdata")]
    connection: PhantomData<sqlite::Connection>,
}

// methods valid in any state
impl<S> CacheIndex<S>
where
    S: IndexState,
{
    /// The [index id](crate::cache::indextype::IndexType) of `self`,
    /// corresponding to the `raw/js5-{index_id}.jcache` file being held.
    #[inline(always)]
    pub fn index_id(&self) -> u32 {
        self.index_id
    }

    /// Returns a view over the [`IndexMetadata`] of `self`.
    #[inline(always)]
    pub fn metadatas(&self) -> &IndexMetadata {
        &(self.metadatas)
    }

    /// Loads the [`Metadata`] of `self`.
    #[allow(unused_variables)]
    #[cfg(not(feature = "mockdata"))]
    fn get_raw_metadata(index_id: u32, connection: &sqlite::Connection) -> CacheResult<Vec<u8>> {
        let encoded_data = {
            let query = "SELECT DATA FROM cache_index";
            let mut statement = connection.prepare(query)?;
            statement.next()?;
            statement.read::<Vec<u8>>(0)?
        };

        #[cfg(feature = "save_mockdata")]
        {
            std::fs::create_dir_all("tests/mockdata".to_string()).unwrap();
            let filename = format!("tests/mockdata/cache_index_{}", index_id);
            let mut file = File::create(&filename).unwrap();
            file.write_all(&encoded_data).unwrap();
        }
        decoder::decompress(encoded_data, None)
    }

    /// Grabs mock data from disk.
    #[cfg(feature = "mockdata")]
    fn get_raw_metadata(index_id: u32) -> CacheResult<Vec<u8>> {
        let filename = format!("tests/mockdata/cache_index_{}", index_id);

        let mut file = File::open(&filename)?;
        let mut encoded_data = Vec::new();
        file.read_to_end(&mut encoded_data)?;

        decoder::decompress(encoded_data, None)
    }

    /// Executes a sql command to retrieve an archive from the cache.
    #[cfg(not(feature = "mockdata"))]
    fn get_file(&self, metadata: &Metadata) -> CacheResult<Vec<u8>> {
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
            std::fs::create_dir_all("tests/mockdata".to_string()).unwrap();
            let filename = format!("tests/mockdata/index_{}_archive_{}", self.index_id(), metadata.archive_id());
            let mut file = File::create(&filename).unwrap();
            file.write_all(&encoded_data).unwrap()
        }
        decoder::decompress(encoded_data, metadata.size())
    }
    /// Grabs mock data from disk.
    #[cfg(feature = "mockdata")]
    pub fn get_file(&self, metadata: &Metadata) -> CacheResult<Vec<u8>> {
        let filename = format!("tests/mockdata/index_{}_archive_{}", self.index_id(), metadata.archive_id());
        let mut file = File::open(&filename)?;
        let mut encoded_data = Vec::new();
        file.read_to_end(&mut encoded_data)?;
        decoder::decompress(encoded_data, metadata.size())
    }

    /// Get an [`Archive`] from `self`.
    ///
    /// # Errors
    ///
    /// Raises [`ArchiveNotFoundError`](CacheError::ArchiveNotFoundError) if `archive_id` is not in `self`.
    pub fn archive(&self, archive_id: u32) -> CacheResult<Archive> {
        let metadata = self
            .metadatas()
            .get(&archive_id)
            .ok_or_else(|| CacheError::ArchiveNotFoundError(self.index_id(), archive_id))?;
        let data = self.get_file(&metadata)?;
        Ok(Archive::deserialize(metadata, data))
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
    pub fn new(index_id: u32) -> CacheResult<CacheIndex<Initial>> {
        let file = match env::var(SYS_VAR) {
            Ok(var) => format!("{}/js5-{}.jcache", var, index_id),
            Err(VarError::NotPresent) => format!("raw/js5-{}.jcache", index_id),
            Err(VarError::NotUnicode(os_str)) => panic!("Unable to parse system variable {}: {:?} as valid unicode.", SYS_VAR, os_str),
        };

        // check if database exists (without creating blank sqlite databases)
        match fs::metadata(&file) {
            Ok(_) => {
                let connection = sqlite::open(file)?;
                let raw_metadata = Self::get_raw_metadata(index_id, &connection)?;
                let metadatas = IndexMetadata::deserialize(index_id, Buffer::new(raw_metadata))?;

                Ok(Self {
                    index_id,
                    metadatas,
                    connection,
                    state: Initial {},
                })
            }
            Err(e) => Err(CacheError::CacheNotFoundError(e, file)),
        }
    }

    /// Mock constructor for `CacheIndex`.
    #[cfg(feature = "mockdata")]
    pub fn new(index_id: u32) -> CacheResult<Self> {
        let raw_metadata = Self::get_raw_metadata(index_id)?;
        let metadatas = IndexMetadata::deserialize(index_id, Buffer::new(raw_metadata))?;

        Ok(CacheIndex {
            index_id: index_id,
            connection: PhantomData,
            metadatas,
            state: Initial {},
        })
    }

    /// Retain only those archives that are in `ids`.
    /// Advances `self` to the `Truncated` state.
    ///
    /// # Panics
    ///
    /// Panics if any of `ids` is not in `self`.
    pub fn retain(self, ids: Vec<u32>) -> CacheIndex<Truncated> {
        let all_ids = self.metadatas().keys().copied().collect::<HashSet<_>>();

        if let Some(missing_id) = ids.iter().find(|id| !all_ids.contains(id)) {
            panic!("Attempted to retain missing archive id {},", missing_id)
        }
        let Self {
            connection,
            index_id,
            metadatas,
            ..
        } = self;

        CacheIndex {
            connection,
            index_id,
            metadatas,
            state: Truncated { feed: ids },
        }
    }

    /// Groups archives according to their surface proximity.
    /// Only valid for the `MAPSV2` index.
    /// Advances `self` to the `Grouped` state.
    ///
    /// # Panics
    ///
    /// Panics if `self.index_id() != IndexType::MAPSV2`.
    pub fn grouped(self, dim_i: RangeInclusive<i32>, dim_j: RangeInclusive<i32>) -> CacheIndex<Grouped> {
        assert_eq!(
            self.index_id(),
            IndexType::MAPSV2,
            "Grouped archives are only valid for IndexType::MAPSV2."
        );

        let Self {
            connection,
            index_id,
            metadatas,
            ..
        } = self;

        CacheIndex {
            connection,
            index_id,
            metadatas,
            state: Grouped { dim_i, dim_j },
        }
    }

    /// Attempt to clone `self`.
    pub fn try_clone(&self) -> CacheResult<Self> {
        Self::new(self.index_id())
    }
}

impl CacheIndex<Truncated> {
    /// Groups archives according to their surface proximity.
    /// Only valid for the `MAPSV2` index.
    /// Advances `self` to the `TruncatedGrouped` state.
    ///
    /// # Panics
    ///
    /// Panics if `self.index_id() != IndexType::MAPSV2`.
    pub fn grouped(self, dim_i: RangeInclusive<i32>, dim_j: RangeInclusive<i32>) -> CacheIndex<TruncatedGrouped> {
        assert_eq!(
            self.index_id(),
            IndexType::MAPSV2,
            "Grouped archives are only valid for IndexType::MAPSV2."
        );

        let Self {
            connection,
            index_id,
            metadatas,
            state,
        } = self;

        CacheIndex {
            connection,
            index_id,
            metadatas,
            state: TruncatedGrouped {
                feed: state.feed,
                dim_i,
                dim_j,
            },
        }
    }
}

impl IntoIterator for CacheIndex<Initial> {
    type Item = Archive;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let feed = self.metadatas().keys().copied().collect::<Vec<u32>>().into_iter();

        IntoIter { index: self, feed }
    }
}

impl IntoIterator for CacheIndex<Truncated> {
    type Item = Archive;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            connection,
            index_id,
            metadatas,
            state,
        } = self;

        let index = CacheIndex {
            connection,
            index_id,
            metadatas,
            state: Initial {},
        };

        IntoIter {
            index,
            feed: state.feed.into_iter(),
        }
    }
}

impl IntoIterator for CacheIndex<Grouped> {
    type Item = ArchiveGroup;

    type IntoIter = IntoIterGrouped;

    fn into_iter(self) -> Self::IntoIter {
        let feed = self.metadatas().keys().copied().collect::<Vec<u32>>().into_iter();
        let dim_i = self.state.dim_i.clone();
        let dim_j = self.state.dim_j.clone();
        IntoIterGrouped {
            index: self,
            feed,
            dim_i,
            dim_j,
        }
    }
}

impl IntoIterator for CacheIndex<TruncatedGrouped> {
    type Item = ArchiveGroup;

    type IntoIter = IntoIterGrouped;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            connection,
            index_id,
            metadatas,
            state,
        } = self;
        let TruncatedGrouped { dim_i, dim_j, feed } = state;
        let index = CacheIndex {
            connection,
            index_id,
            metadatas,
            state: Grouped {
                dim_i: dim_i.clone(),
                dim_j: dim_j.clone(),
            },
        };

        IntoIterGrouped {
            index,
            dim_i,
            dim_j,
            feed: feed.into_iter(),
        }
    }
}

/// Iterator over all [`Archive`]s of `self`. Yields in arbitrary order.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntoIter {
    index: CacheIndex<Initial>,
    feed: std::vec::IntoIter<u32>,
}

impl IntoIter {
    /// Returns a view of the [`IndexMetadata`] of the contained [`CacheIndex`].
    pub fn metadatas(&self) -> &IndexMetadata {
        self.index.metadatas()
    }
}

impl Iterator for IntoIter {
    type Item = Archive;

    fn next(&mut self) -> Option<Self::Item> {
        self.feed.next().map(|archive_id| {
            self.index
                .archive(archive_id)
                .unwrap_or_else(|_| panic!("Error decoding index {} archive {}.", self.index.index_id(), archive_id))
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.feed.size_hint()
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator of [`ArchiveGroup`]s. Used internally by renderers that need to know about the surrounding [`GroupMapSquare`](crate::definitions::mapsquares::GroupMapSquare).
pub struct IntoIterGrouped {
    /// Handle to the underlying [`CacheIndex`].
    index: CacheIndex<Grouped>,
    feed: std::vec::IntoIter<u32>,
    /// The horizontal range of the [`ArchiveGroup`]s.
    dim_i: RangeInclusive<i32>,
    /// The vertical range of the [`ArchiveGroup`]s.
    dim_j: RangeInclusive<i32>,
}

impl Iterator for IntoIterGrouped {
    type Item = ArchiveGroup;

    fn next(&mut self) -> Option<Self::Item> {
        self.feed.next().map(|core_id| {
            let (i, j) = ((core_id & 0x7F) as i32, (core_id >> 7) as i32);

            let group_ids = iproduct!(self.dim_i.clone(), self.dim_j.clone())
                .map(|(di, dj)| (i + di, j + dj))
                .filter(|(i, j)| *i >= 0 && *j >= 0)
                .map(|(i, j)| (i + (j << 7)) as u32);

            let archives = group_ids.filter_map(|archive_id| self.index.archive(archive_id).ok()).collect::<Vec<_>>();

            ArchiveGroup::new(core_id, archives)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.feed.size_hint()
    }
}

/// Asserts whether all indices' metadata match their contents.
/// Indices 14, 40, 54, 55 are not necessarily complete.
///
/// Exposed as `--assert coherence`.
///
/// # Panics
/// Panics if compiled with feature `mockdata`.
#[cfg(not(any(feature = "mockdata", feature = "save_mockdata")))]
pub fn assert_coherence() -> CacheResult<()> {
    for index_id in IndexType::iterator() {
        match CacheIndex::new(index_id)?.assert_coherence() {
            Ok(_) => println!("Index {} is coherent!", index_id),
            Err(e) => println!("Index {} is not coherent: {} and possibly others.", index_id, e),
        }
    }
    Ok(())
}
