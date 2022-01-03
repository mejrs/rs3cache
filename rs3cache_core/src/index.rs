//! The interface between [rs3cache](crate) and the cache database.

#![allow(unused_imports)] // varies based on mock config flags

/// This contains the game-specific implementations.
#[cfg_attr(feature = "sqlite", path = "index/sqlite.rs")]
#[cfg_attr(feature = "dat2", path = "index/dat2.rs")]
#[cfg_attr(feature = "dat", path = "index/dat.rs")]
mod index_impl;

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
pub use index_impl::*;
use itertools::iproduct;
use path_macro::path;

#[cfg(feature = "dat2")]
use crate::xtea::Xtea;
use crate::{
    arc::Archive,
    buf::BufExtra,
    decoder,
    error::{CacheError, CacheResult},
    indextype::IndexType,
    meta::{IndexMetadata, Metadata},
};

mod states {
    use std::ops::RangeInclusive;

    /// Initial state of [`CacheIndex`](super::CacheIndex).
    pub struct Initial {}

    pub struct Truncated {
        pub feed: Vec<u32>,
    }

    /// Trait that describes the current index state. Cannot be implemented.
    pub trait IndexState {}
    impl IndexState for Initial {}
    impl IndexState for Truncated {}
}

pub use states::{IndexState, Initial, Truncated};

/// Container of [`Archive`]s.
pub struct CacheIndex<S: IndexState> {
    index_id: u32,
    metadatas: IndexMetadata,
    state: S,
    path: PathBuf,

    #[cfg(feature = "sqlite")]
    connection: sqlite::Connection,

    #[cfg(any(feature = "dat2", feature = "dat"))]
    file: Box<[u8]>,

    #[cfg(feature = "dat2")]
    xteas: Option<HashMap<u32, Xtea>>,
}

// methods valid in any state
impl<S> CacheIndex<S>
where
    S: IndexState,
{
    /// The [index id](crate::indextype::IndexType) of `self`,
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

    /// Get an [`Archive`] from `self`.
    ///
    /// # Errors
    ///
    /// Raises [`ArchiveNotFoundError`](CacheError::ArchiveNotFoundError) if `archive_id` is not in `self`.
    #[cfg(any(feature = "sqlite", feature = "dat2"))]
    pub fn archive(&self, archive_id: u32) -> CacheResult<Archive> {
        let metadata = self
            .metadatas()
            .get(&archive_id)
            .ok_or_else(|| CacheError::ArchiveNotFoundError(self.index_id(), archive_id))?;
        let data = self.get_file(metadata)?;

        Ok(Archive::deserialize(metadata, data))
    }

    #[cfg(feature = "dat")]
    pub fn archive(&self, archive_id: u32) -> CacheResult<Archive> {
        // FIXME
        let metadata = Metadata {
            index_id: self.index_id,
            archive_id,
            child_count: 1,
            child_indices: vec![0],
            ..Default::default()
        };

        let data = self.get_file(&metadata)?;

        if self.index_id == 0 {
            Ok(Archive::deserialize_jag(&metadata, data)?)
        } else {
            Ok(Archive {
                index_id: self.index_id,
                archive_id,
                files: {
                    let mut files = BTreeMap::new();
                    files.insert(metadata.child_indices()[0], data);
                    files
                },
                files_named: BTreeMap::new(),
            })
        }
    }
}

impl CacheIndex<Initial> {
    /// Retain only those archives that are in `ids`.
    /// Advances `self` to the `Truncated` state.
    ///
    /// # Panics
    ///
    /// Panics if any of `ids` is not in `self`.
    pub fn retain(self, ids: Vec<u32>) -> CacheIndex<Truncated> {
        let all_ids = self.metadatas().keys().copied().collect::<BTreeSet<_>>();

        if let Some(missing_id) = ids.iter().find(|id| !all_ids.contains(id)) {
            panic!("Attempted to retain missing archive id {},", missing_id)
        }
        let Self {
            path,
            #[cfg(feature = "sqlite")]
            connection,
            #[cfg(any(feature = "dat2", feature = "dat"))]
            file,
            index_id,
            metadatas,
            #[cfg(feature = "dat2")]
            xteas,
            ..
        } = self;

        CacheIndex {
            path,
            #[cfg(feature = "sqlite")]
            connection,
            #[cfg(any(feature = "dat2", feature = "dat"))]
            file,
            index_id,
            metadatas,
            #[cfg(feature = "dat2")]
            xteas,
            state: Truncated { feed: ids },
        }
    }
}

impl IntoIterator for CacheIndex<Initial> {
    type Item = CacheResult<Archive>;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let feed = self.metadatas().keys().copied().collect::<Vec<u32>>().into_iter();

        IntoIter { index: self, feed }
    }
}

impl IntoIterator for CacheIndex<Truncated> {
    type Item = CacheResult<Archive>;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            path,
            #[cfg(feature = "sqlite")]
            connection,
            #[cfg(any(feature = "dat2", feature = "dat"))]
            file,
            index_id,
            metadatas,
            #[cfg(feature = "dat2")]
            xteas,
            state,
        } = self;

        let index = CacheIndex {
            path,
            #[cfg(feature = "sqlite")]
            connection,
            #[cfg(any(feature = "dat2", feature = "dat"))]
            file,
            index_id,
            metadatas,
            #[cfg(feature = "dat2")]
            xteas,
            state: Initial {},
        };

        IntoIter {
            index,
            feed: state.feed.into_iter(),
        }
    }
}

/// Iterator over all [`Archive`]s of `self`. Yields in arbitrary order.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntoIter {
    pub(crate) index: CacheIndex<Initial>,
    feed: std::vec::IntoIter<u32>,
}

impl IntoIter {
    /// Returns a view of the [`IndexMetadata`] of the contained [`CacheIndex`].
    pub fn metadatas(&self) -> &IndexMetadata {
        self.index.metadatas()
    }
}

impl Iterator for IntoIter {
    type Item = CacheResult<Archive>;

    fn next(&mut self) -> Option<Self::Item> {
        self.feed.next().map(|archive_id| self.index.archive(archive_id))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.feed.size_hint()
    }
}
