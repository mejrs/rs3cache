//! Units of data in a [`CacheIndex`](crate::index::CacheIndex).
//!
//! Each [`Archive`] conatins files, which contain the actual data that can be parsed with
//! the appropriate deserializer in [`definitions`](../../rs3cache/definitions/index.html).
//!
//! None of the structs in this module can be constructed directly.
//! Instead, construct a [`CacheIndex`](crate::index::CacheIndex)
//! and use its [`IntoIterator`] implementation or its [`archive`](crate::index::CacheIndex::archive())
//! method instead.

use std::collections::{BTreeMap, HashSet};

use bytes::{Buf, Bytes};
use itertools::izip;
#[cfg(feature = "pyo3")]
use pyo3::{exceptions::PyKeyError, prelude::*, types::PyBytes};

use crate::{
    buf::BufExtra,
    error::{CacheError, CacheResult},
    meta::Metadata,
};

/// A collection of files.
#[cfg_eval]
#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Clone, Default)]
pub struct Archive {
    index_id: u32,
    archive_id: u32,
    files: BTreeMap<u32, Bytes>,
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
    /// The [`index id`](crate::indextype::IndexType) of `self`.
    pub const fn index_id(&self) -> u32 {
        self.index_id
    }

    /// The archive id of `self`.
    pub const fn archive_id(&self) -> u32 {
        self.archive_id
    }

    pub(crate) fn deserialize(metadata: &Metadata, data: Bytes) -> Archive {
        let index_id = metadata.index_id();
        let archive_id = metadata.archive_id();
        let files = match metadata.child_count() {
            0 => unreachable!(),
            1 => {
                let mut files = BTreeMap::new();
                files.insert(metadata.child_indices()[0], data);
                files
            }

            #[cfg(feature = "rs3")]
            child_count => {
                use crate::utils::adapters::Pairwisor;

                assert_eq!(data[0], 1);

                let mut offset_data = data.slice(1..((child_count + 1) * 4 + 1) as usize);

                let offsets = std::iter::repeat_with(|| offset_data.get_i32() as usize)
                    .take(child_count as usize + 1)
                    .pairwise();

                let files = izip!(metadata.child_indices(), offsets)
                    .map(|(i, (start, end))| (*i, data.slice(start..end)))
                    .collect::<BTreeMap<_, _>>();

                files
            }

            #[cfg(any(feature = "osrs", feature = "legacy"))]
            child_count => {
                use crate::utils::adapters::Accumulator;
                let mut data = data;

                let n_chunks = *data.last().unwrap() as usize;

                let offset_start = data.len().checked_sub(4 * n_chunks * (child_count as usize) + 1).unwrap();
                let mut offset_data = data.split_off(offset_start);

                let offsets = std::iter::repeat_with(|| offset_data.get_i32())
                    .take(child_count as usize)
                    .accumulate(|x, y| x + y);

                let files = izip!(metadata.child_indices(), offsets)
                    .map(|(i, n)| (*i, data.split_to(n.try_into().unwrap())))
                    .collect::<BTreeMap<_, _>>();
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
    pub fn take_file(&mut self, file_id: &u32) -> CacheResult<Bytes> {
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
    pub fn take_files(self) -> BTreeMap<u32, Bytes> {
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

#[cfg(feature = "pyo3")]
#[pymethods]
impl Archive {
    fn file<'p>(&self, py: Python<'p>, file_id: u32) -> PyResult<&'p PyBytes> {
        if let Some(file) = self.files.get(&file_id) {
            Ok(PyBytes::new(py, file))
        } else {
            Err(PyKeyError::new_err(format!("File {} is not present.", file_id)))
        }
    }

    fn files<'p>(&self, py: Python<'p>) -> PyResult<BTreeMap<u32, &'p PyBytes>> {
        Ok(self.files.iter().map(|(&id, file)| (id, PyBytes::new(py, file))).collect())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Archive({}, {})", self.index_id(), self.archive_id()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Archive({}, {})", self.index_id(), self.archive_id()))
    }
}
