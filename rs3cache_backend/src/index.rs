//! The interface between [rs3cache](crate) and the cache database.

#![allow(unused_imports)] // varies based on mock config flags
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env::{self, VarError},
    fmt,
    fs::{self, File},
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom, Write},
    marker::PhantomData,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    sync::Arc,
};

use ::error::Context;
use bytes::{Buf, Bytes};
use itertools::iproduct;
use path_macro::path;

#[cfg(feature = "dat2")]
use crate::xtea::Xtea;
use crate::{
    arc::Archive,
    buf::BufExtra,
    decoder,
    error::{self, CacheError, CacheResult},
    meta::{IndexMetadata, Metadata},
};

/// This contains the game-specific implementations.
#[cfg_attr(feature = "sqlite", path = "index/sqlite.rs")]
#[cfg_attr(feature = "dat2", path = "index/dat2.rs")]
#[cfg_attr(feature = "dat", path = "index/dat.rs")]
mod index_impl;

pub use index_impl::*;

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
    input: Arc<CachePath>,

    #[cfg(feature = "sqlite")]
    connection: rusqlite::Connection,

    #[cfg(any(feature = "dat2", feature = "dat"))]
    file: File,

    #[cfg(feature = "dat2")]
    xteas: Option<HashMap<u32, Xtea>>,
}

#[derive(Clone, Debug, Default)]
pub enum CachePath {
    #[default]
    Omitted,
    Env(PathBuf),
    Given(PathBuf),
}

impl fmt::Display for CachePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.as_ref();
        let path = ::path_absolutize::Absolutize::absolutize(path).unwrap_or(std::borrow::Cow::Borrowed(path));
        fmt::Display::fmt(&path.display(), f)
    }
}

impl AsRef<Path> for CachePath {
    fn as_ref(&self) -> &Path {
        match self {
            CachePath::Omitted => Path::new(""),
            CachePath::Env(p) | CachePath::Given(p) => p,
        }
    }
}

pub struct LocationHelp<'p>(&'p CachePath);

impl fmt::Display for LocationHelp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            CachePath::Given(path) => writeln!(f, "looking in this location because the path {path:?} was given as an argument")?,
            CachePath::Env(path) => writeln!(
                f,
                "looking in this location because the path {path:?} was retrieved from an environment variable"
            )?,
            CachePath::Omitted => writeln!(f, "looking in the current directory because no path was given")?,
        }

        Ok(())
    }
}

// methods valid in any state
impl<S> CacheIndex<S>
where
    S: IndexState,
{
    /// The index id of `self`,
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
            .context(ArchiveMissing {
                index_id: self.index_id,
                archive_id,
            })
            .context(error::Integrity)?;
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
            panic!("Attempted to retain missing archive id {missing_id},")
        }
        let Self {
            input,
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
            input,
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
            input,
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
            input,
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

impl ExactSizeIterator for IntoIter {}

#[derive(::error::Error)]
pub enum IntegrityError {
    #[error = "cannot open cache"]
    #[help = "expecting the following folder structure: \n   {input}{STRUCTURE}"]
    #[help = "{LocationHelp(input)}"]
    CannotOpen {
        #[cfg(feature = "sqlite")]
        #[source]
        source: rusqlite::Error,
        file: PathBuf,
        input: Arc<CachePath>,
    },
    #[error = "Index {index_id} does not contain archive {archive_id}"]
    ArchiveMissing { index_id: u32, archive_id: u32 },
    #[error = "Index {index_id} does not contain archive {name}"]
    ArchiveMissingNamed { index_id: u32, name: String },
    #[error = "Index {index_id}, archive {archive_id} does not contain file {file} "]
    FileMissing { index_id: u32, archive_id: u32, file: u32 },
    #[error = "Index {index_id}, archive {archive_id} does not contain file {name} "]
    FileMissingNamed { index_id: u32, archive_id: u32, name: String },
    #[cfg(feature = "dat2")]
    #[error = "Mapsquare ({i}, {i}) has no xtea"]
    XteaMissing { i: u32, j: u32 },
    #[error = "Index {metadata.index_id} Archive {metadata.archive_id}: Crc does not match: {crc} !=  {metadata.crc}"]
    Crc { crc: i64, metadata: Metadata },
    #[error = "Index {metadata.index_id} Archive {metadata.archive_id}: Version does not match: {version} !=  {metadata.version}"]
    Version { version: i64, metadata: Metadata },
    #[error = "Index {metadata.index_id}'s archive {metadata.archive_id} is blank"]
    Blank { metadata: Metadata },
    #[error = "Error retrieving {metadata}"]
    Corrupted {
        #[cfg(feature = "sqlite")]
        #[source]
        source: rusqlite::Error,
        metadata: Metadata,
    },
    #[error = "Something went wrong"]
    Other {
        #[cfg(feature = "sqlite")]
        #[source]
        source: rusqlite::Error,
    },
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
