//! Metadata about the cache itself.

use std::{
    collections::{
        btree_map::{IntoIter, Iter, Keys},
        BTreeMap,
    },
    iter,
    ops::Add,
};

use bytes::{Buf, Bytes};
use itertools::izip;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Serialize, Serializer};

use crate::{buf::BufExtra, error::CacheResult, utils::adapters::Accumulator};

/// Metadata about [`Archive`](crate::arc::Archive)s.

#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct Metadata {
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    index_id: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    archive_id: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    name: Option<i32>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    crc: i32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    version: i32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    unknown: Option<i32>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    compressed_size: Option<u32>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    size: Option<u32>,
    #[serde(serialize_with = "bytes_to_vec")]
    digest: Option<Bytes>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    child_count: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    child_indices: Vec<u32>,
}

#[cfg(feature = "pyo3")]
#[pyo3::pymethods]
impl Metadata {
    #[getter(digest)]
    fn py_digest(&self, py: Python) -> Py<PyAny> {
        match self.digest {
            Some(ref b) => pyo3::types::PyBytes::new(py, b).into(),
            None => py.None(),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Metadata({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Metadata({})", serde_json::to_string(self).unwrap()))
    }
}

pub fn bytes_to_vec<S>(bytes: &Option<Bytes>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match *bytes {
        Some(ref value) => s.serialize_some(&**value),
        None => s.serialize_none(),
    }
}

impl Metadata {
    /// The [index id](crate::indextype::IndexType) of this Metadata.
    #[inline(always)]
    pub const fn index_id(&self) -> u32 {
        self.index_id
    }

    /// The archive_id of the [`Archive`](crate::arc::Archive).
    #[inline(always)]
    pub const fn archive_id(&self) -> u32 {
        self.archive_id
    }

    /// The hashed name of the [`Archive`](crate::arc::Archive), if present.
    #[inline(always)]
    pub const fn name(&self) -> Option<i32> {
        self.name
    }

    /// See [CRC](https://en.wikipedia.org/wiki/Cyclic_redundancy_check).
    #[inline(always)]
    pub const fn crc(&self) -> i32 {
        self.crc
    }

    /// Usually, the amount of seconds between [Unix Epoch](https://en.wikipedia.org/wiki/Unix_time)
    /// and when the [`Archive`](crate::arc::Archive) was compiled,
    /// but it can also be a version counter.
    #[inline(always)]
    pub const fn version(&self) -> i32 {
        self.version
    }

    #[allow(missing_docs)]
    #[inline(always)]
    pub const fn unknown(&self) -> Option<i32> {
        self.unknown
    }

    /// Size of the [`Archive`](crate::arc::Archive).
    #[inline(always)]
    pub const fn compressed_size(&self) -> Option<u32> {
        self.compressed_size
    }

    /// Size of the [`Archive`](crate::arc::Archive) once decompressed.
    #[inline(always)]
    pub const fn size(&self) -> Option<u32> {
        self.size
    }

    /// See [whirlpool digest](https://en.wikipedia.org/wiki/Whirlpool_(hash_function) ).
    #[inline(always)]
    pub fn digest(&self) -> Option<&[u8]> {
        self.digest.as_deref()
    }

    /// The count of files in `self`.
    #[inline(always)]
    pub const fn child_count(&self) -> u32 {
        self.child_count
    }

    /// Enumerated file ids of files in the [`Archive`](crate::arc::Archive).
    #[inline(always)]
    pub fn child_indices(&self) -> &[u32] {
        &self.child_indices
    }
}

/// Contains the [`Metadata`] for every [`Archive`](crate::arc::Archive) in the index.
#[cfg_eval]
#[allow(missing_docs)]
#[derive(Serialize, Clone, Debug, Default)]
pub struct IndexMetadata {
    metadatas: BTreeMap<u32, Metadata>,
}

impl IndexMetadata {
    #[cfg(feature = "osrs")]
    pub(crate) fn empty() -> Self {
        Self {
            metadatas: BTreeMap::default(),
        }
    }
    /// Returns the ids of the archives in the index.
    #[inline(always)]
    pub fn keys(&self) -> Keys<'_, u32, Metadata> {
        self.metadatas.keys()
    }

    /// Constructor for [`IndexMetadata`]. `index_id` must be one of [`IndexType`](rs3cache_core::indextype::IndexType).
    pub(crate) fn deserialize(index_id: u32, mut buffer: Bytes) -> CacheResult<Self> {
        let format = buffer.get_i8();
        let _index_utc_stamp = if format > 5 { Some(buffer.get_i32()) } else { None };
        let [named, hashed, unk4, ..] = buffer.get_bitflags();
        let entry_count = if format >= 7 {
            buffer.get_smart32().unwrap() as usize
        } else {
            buffer.get_u16() as usize
        };

        let archive_ids = iter::repeat_with(|| {
            if format >= 7 {
                buffer.get_smart32().unwrap()
            } else {
                buffer.get_u16() as u32
            }
        })
        .take(entry_count)
        .accumulate(Add::add)
        .collect::<Vec<u32>>();

        let names = if named {
            iter::repeat_with(|| Some(buffer.get_i32()))
                .take(entry_count)
                .collect::<Vec<Option<i32>>>()
        } else {
            vec![None; entry_count]
        };

        let crcs = iter::repeat_with(|| buffer.get_i32()).take(entry_count).collect::<Vec<i32>>();

        let unknowns = if unk4 {
            iter::repeat_with(|| Some(buffer.get_i32()))
                .take(entry_count)
                .collect::<Vec<Option<i32>>>()
        } else {
            vec![None; entry_count]
        };

        let digests = if hashed {
            iter::repeat_with(|| Some(buffer.copy_to_bytes(64))).take(entry_count).collect()
        } else {
            vec![None; entry_count]
        };

        let (compressed_sizes, sizes): (Vec<_>, Vec<_>) = if unk4 {
            iter::repeat_with(|| (Some(buffer.get_u32()), Some(buffer.get_u32())))
                .take(entry_count)
                .unzip()
        } else {
            (vec![None; entry_count], vec![None; entry_count])
        };

        let versions = iter::repeat_with(|| buffer.get_i32()).take(entry_count).collect::<Vec<i32>>();

        let child_counts = iter::repeat_with(|| {
            if format >= 7 {
                buffer.get_smart32().unwrap()
            } else {
                buffer.get_u16() as u32
            }
        })
        .take(entry_count)
        .collect::<Vec<u32>>();

        let child_indices = child_counts
            .iter()
            .map(|count| {
                iter::repeat_with(|| {
                    if format >= 7 {
                        buffer.get_smart32().unwrap()
                    } else {
                        buffer.get_u16() as u32
                    }
                })
                .take(*count as usize)
                .accumulate(Add::add)
                .collect::<Vec<u32>>()
            })
            .collect::<Vec<Vec<u32>>>();

        let metadatas = izip!(
            archive_ids,
            names,
            crcs,
            unknowns,
            digests,
            compressed_sizes,
            sizes,
            versions,
            child_counts,
            child_indices
        )
        .map(
            |(archive_id, name, crc, unknown, digest, compressed_size, size, version, child_count, child_indices)| {
                (
                    archive_id,
                    Metadata {
                        index_id,
                        archive_id,
                        name,
                        crc,
                        version,
                        unknown,
                        compressed_size,
                        size,
                        digest,
                        child_count,
                        child_indices,
                    },
                )
            },
        )
        .collect();

        //assert!(!buffer.buf.has_remaining());

        Ok(Self { metadatas })
    }

    /// View a specific [`Metadata`] of `self`.
    #[inline(always)]
    pub fn get(&self, archive_id: &u32) -> Option<&Metadata> {
        self.metadatas.get(archive_id)
    }

    /// An iterator visiting all key-value pairs in arbitrary order. The iterator element type is `(&'a u32, &'a Metadata)`.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, u32, Metadata> {
        self.metadatas.iter()
    }

    /// Get a reference to the index metadata's metadatas.
    pub fn metadatas(&self) -> &BTreeMap<u32, Metadata> {
        &self.metadatas
    }
}

impl IntoIterator for IndexMetadata {
    type Item = (u32, Metadata);

    type IntoIter = IntoIter<u32, Metadata>;

    fn into_iter(self) -> Self::IntoIter {
        self.metadatas.into_iter()
    }
}
