//! Metadata about the cache itself.

use std::{
    collections::{
        btree_map::{IntoIter, Iter, Keys},
        BTreeMap,
    },
    fmt,
};

use bytes::Bytes;
use serde::{Serialize, Serializer};
#[cfg(feature = "dat2")]
use {crate::buf::BufExtra, crate::buf::ReadError, bytes::Buf, rs3cache_utils::adapters::Accumulator, std::iter::repeat_with, std::ops::Add};
#[cfg(feature = "sqlite")]
use {crate::buf::BufExtra, crate::buf::ReadError, bytes::Buf, rs3cache_utils::adapters::Accumulator, std::iter::repeat_with, std::ops::Add};
#[cfg(feature = "pyo3")]
use {
    pyo3::class::basic::CompareOp,
    pyo3::prelude::*,
    std::collections::hash_map::DefaultHasher,
    std::hash::{Hash, Hasher},
};

/// Metadata about [`Archive`](crate::arc::Archive)s.
#[cfg_eval]
#[cfg_attr(feature = "pyo3", pyclass(frozen))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default, Hash, Eq, PartialOrd, Ord, PartialEq)]
pub struct Metadata {
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub index_id: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub archive_id: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub name: Option<i32>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub crc: i32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub version: i32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub unknown: Option<i32>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub compressed_size: Option<u32>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub size: Option<u32>,

    // Getter in pymethods block below
    #[serde(serialize_with = "bytes_to_vec")]
    pub digest: Option<Bytes>,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub child_count: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub child_indices: Vec<u32>,
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Metadata({})", serde_json::to_string(self).unwrap())
    }
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

    fn __hash__(&self) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn __richcmp__(&self, other: &Metadata, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => Ok(self < other),
            CompareOp::Le => Ok(self <= other),
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            CompareOp::Gt => Ok(self > other),
            CompareOp::Ge => Ok(self >= other),
        }
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

#[derive(Serialize, Clone, Debug, Default, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub struct IndexMetadata {
    metadatas: BTreeMap<u32, Metadata>,
}

impl IndexMetadata {
    #[cfg(any(feature = "dat2", feature = "dat"))]
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

    /// Constructor for [`IndexMetadata`]. `index_id` must be one of [`IndexType`](rs3cache_backend::indextype::IndexType).
    #[cfg(any(feature = "sqlite", feature = "dat2"))]
    pub(crate) fn deserialize(index_id: u32, mut buffer: Bytes) -> Result<Self, ReadError> {
        let format = buffer.try_get_i8()?;

        let _index_utc_stamp = if format > 5 { Some(buffer.try_get_i32()?) } else { None };

        let [named, hashed, sized, ..] = buffer.get_bitflags();

        let entry_count = if format >= 7 {
            buffer.try_get_smart32()?.unwrap() as usize
        } else {
            buffer.try_get_u16()? as usize
        };

        let archive_ids = repeat_with(|| try {
            if format >= 7 {
                buffer.try_get_smart32()?.unwrap()
            } else {
                buffer.try_get_u16()? as u32
            }
        })
        .take(entry_count)
        .collect::<Result<Vec<u32>, ReadError>>()?
        .into_iter()
        .accumulate(Add::add)
        .collect::<Vec<u32>>();

        let names = if named {
            repeat_with(|| try { Some(buffer.try_get_i32()?) })
                .take(entry_count)
                .collect::<Result<Vec<Option<i32>>, ReadError>>()?
        } else {
            vec![None; entry_count]
        };

        let crcs = repeat_with(|| buffer.try_get_i32())
            .take(entry_count)
            .collect::<Result<Vec<i32>, ReadError>>()?;

        let unknowns = if cfg!(feature = "sqlite") && sized {
            repeat_with(|| try { Some(buffer.try_get_i32()?) })
                .take(entry_count)
                .collect::<Result<Vec<Option<i32>>, ReadError>>()?
        } else {
            vec![None; entry_count]
        };

        let digests = if hashed {
            repeat_with(|| Some(buffer.copy_to_bytes(64))).take(entry_count).collect()
        } else {
            vec![None; entry_count]
        };

        let (compressed_sizes, sizes): (Vec<_>, Vec<_>) = if sized {
            repeat_with(|| (Some(buffer.get_u32()), Some(buffer.get_u32()))).take(entry_count).unzip()
        } else {
            (vec![None; entry_count], vec![None; entry_count])
        };

        let versions = repeat_with(|| buffer.get_i32()).take(entry_count).collect::<Vec<i32>>();

        let child_counts = repeat_with(|| {
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
                repeat_with(|| {
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

        let metadatas = itertools::izip!(
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

        debug_assert!(!buffer.has_remaining());

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
