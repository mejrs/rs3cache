//! Units of data in a [`CacheIndex`](crate::cache::index::CacheIndex).
//! 
//! Each [`Archive`] conatins files, which contain the actual data that can be parsed with
//! the appropriate deserializer in [`definitions`](crate::definitions).
//!
//! None of the structs in this module can be constructed directly.
//! Instead, construct a [`CacheIndex`](crate::cache::index::CacheIndex)
//! and use its [`IntoIterator`] implementation or its [`archive`](crate::cache::index::CacheIndex::archive())
//! method instead.

use crate::{
    cache::{buf::Buffer, meta::Metadata},
    utils::{
        adapters::Pairwisor,
        error::{CacheError, CacheResult},
    },
};

use std::collections::HashMap;
use std::collections::HashSet;

use itertools::izip;

/// A group of archives.
pub struct ArchiveGroup {
    core_id: u32,

    archives: Vec<Archive>,
}

impl ArchiveGroup {
    pub(crate) fn new(core_id: u32, archives: Vec<Archive>) -> Self {
        Self { core_id, archives }
    }

    /// Get the Archive id of the [`ArchiveGroup`].
    #[inline(always)]
    pub const fn core_id(&self) -> u32 {
        self.core_id
    }
}

type File = Vec<u8>;

impl IntoIterator for ArchiveGroup {
    type Item = Archive;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.archives.into_iter()
    }
}

/// A collection of files.
pub struct Archive {
    index_id: u32,
    archive_id: u32,
    files: HashMap<u32, File>,
    poison_state: HashSet<u32>,
}

impl std::fmt::Debug for Archive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archive")
            .field("index_id", &self.index_id)
            .field("archive_id", &self.archive_id)
            .field("files", &self.files.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Archive {
    /// The [`index id`](crate::cache::indextype::IndexType) of `self`.
    pub const fn index_id(&self) -> u32 {
        self.index_id
    }

    /// The archive id of `self`.
    pub const fn archive_id(&self) -> u32 {
        self.archive_id
    }

    pub(crate) fn deserialize(metadata: &Metadata, data: File) -> Archive {
        let index_id = metadata.index_id();
        let archive_id = metadata.archive_id();
        let files = match metadata.child_count() {
            0 => unreachable!(),
            1 => {
                let mut files = HashMap::new();
                files.insert(metadata.child_indices()[0], data);
                files
            }
            child_count => {
                let mut buffer = Buffer::new(data);

                let first = buffer.read_unsigned_byte();
                assert_eq!(first, 1);
                let offsets = std::iter::repeat_with(|| buffer.read_unsigned_int())
                    .take(child_count as usize + 1)
                    .pairwise()
                    .collect::<Vec<_>>();

                let files = izip!(metadata.child_indices(), offsets)
                    .map(|(i, (start, end))| (*i, buffer.read_n_bytes((end - start) as usize)))
                    .collect::<HashMap<_, _>>();

                assert_eq!(buffer.remaining(), 0);
                files
            }
        };

        Archive {
            index_id,
            archive_id,
            files,
            poison_state: HashSet::new(),
        }
    }

    /// Removes and returns a File.
    ///
    /// # Panics
    ///
    /// This function will panic if file `file_id` has already been removed.
    pub fn take_file(&mut self, file_id: &u32) -> CacheResult<File> {
        if self.poison_state.insert(*file_id) {
            self.files
                .remove(file_id)
                .ok_or_else(|| CacheError::FileNotFoundError(self.index_id(), self.archive_id(), *file_id))
        } else {
            panic!("PoisonError: file {} has already been withdrawn.", file_id)
        }
    }

    /// Take the files. Consumes the [`Archive`].
    ///
    /// # Panics
    ///
    /// This function will panic if [`take_file`](Archive::take_file) has been called prior.
    pub fn take_files(self) -> HashMap<u32, File> {
        if self.poison_state.is_empty() {
            self.files
        } else {
            panic!("PoisonError: some files have already been withdrawn")
        }
    }

    /// The quantity of files currently in the archive.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}
