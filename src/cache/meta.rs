//! Metadata about the cache itself.

use crate::{
    cache::buf::Buffer,
    utils::{adapters::Accumulator, error::CacheResult},
};
use itertools::izip;
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;
use std::{
    collections::{hash_map, HashMap},
    iter,
    ops::Add,
};

/// Metadata about [`Archive`](crate::cache::arc::Archive)s.
#[pyclass]
#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
    #[pyo3(get)]
    index_id: u32,

    #[pyo3(get)]
    archive_id: u32,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<u32>,

    #[pyo3(get)]
    crc: i32,

    #[pyo3(get)]
    version: i32,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    unknown: Option<i32>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    compressed_size: Option<u32>,

    #[pyo3(get)]
    size: Option<u32>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<Vec<u8>>,

    #[pyo3(get)]
    child_count: u32,

    #[pyo3(get)]
    child_indices: Vec<u32>,
}

impl Metadata {
    /// The [index id](crate::cache::indextype::IndexType) of this Metadata.
    #[inline(always)]
    pub const fn index_id(&self) -> u32 {
        self.index_id
    }

    /// The archive_id of the [`Archive`](crate::cache::arc::Archive).
    #[inline(always)]
    pub const fn archive_id(&self) -> u32 {
        self.archive_id
    }

    /// The hashed name of the [`Archive`](crate::cache::arc::Archive), if present.
    #[inline(always)]
    pub const fn name(&self) -> Option<u32> {
        self.name
    }

    /// See [CRC](https://en.wikipedia.org/wiki/Cyclic_redundancy_check).
    #[inline(always)]
    pub const fn crc(&self) -> i32 {
        self.crc
    }

    /// Usually, the amount of seconds between [Unix Epoch](https://en.wikipedia.org/wiki/Unix_time)
    /// and when the [`Archive`](crate::cache::arc::Archive) was compiled,
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

    /// Size of the [`Archive`](crate::cache::arc::Archive).
    #[inline(always)]
    pub const fn compressed_size(&self) -> Option<u32> {
        self.compressed_size
    }

    /// Size of the [`Archive`](crate::cache::arc::Archive) once decompressed.
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

    /// Enumerated file ids of files in the [`Archive`](crate::cache::arc::Archive).
    #[inline(always)]
    pub fn child_indices(&self) -> &[u32] {
        &self.child_indices
    }
}

/// Contains the [`Metadata`] for every [`Archive`](crate::cache::arc::Archive) in the index.
pub struct IndexMetadata {
    metadatas: HashMap<u32, Metadata>,
}

impl IndexMetadata {
    /// Returns the ids of the archives in the index.
    #[inline(always)]
    pub fn keys(&self) -> hash_map::Keys<'_, u32, Metadata> {
        self.metadatas.keys()
    }

    /// Constructor for [`IndexMetadata`]. `index_id` must be one of [`IndexType`](crate::cache::indextype::IndexType).
    ///
    pub(crate) fn deserialize(index_id: u32, mut buffer: Buffer) -> CacheResult<Self> {
        let format = buffer.read_byte();
        let _index_utc_stamp = if format > 5 { Some(buffer.read_int()) } else { None };
        let [named, hashed, unk4, ..] = buffer.read_bitflags();
        let entry_count = if format >= 7 {
            buffer.read_smart32()? as usize
        } else {
            buffer.read_unsigned_short() as usize
        };

        let archive_ids = iter::repeat_with(|| {
            Ok({
                if format >= 7 {
                    buffer.read_smart32()?
                } else {
                    buffer.read_unsigned_short() as u32
                }
            })
        })
        .take(entry_count)
        .collect::<CacheResult<Vec<u32>>>()?
        .into_iter()
        .accumulate(Add::add)
        .collect::<Vec<u32>>();

        let names = if named {
            iter::repeat_with(|| Some(buffer.read_unsigned_int()))
                .take(entry_count)
                .collect::<Vec<Option<u32>>>()
        } else {
            vec![None; entry_count]
        };

        let crcs = iter::repeat_with(|| buffer.read_int()).take(entry_count).collect::<Vec<i32>>();

        let unknowns = if unk4 {
            iter::repeat_with(|| Some(buffer.read_int()))
                .take(entry_count)
                .collect::<Vec<Option<i32>>>()
        } else {
            vec![None; entry_count]
        };

        let digests: Vec<Option<Vec<u8>>> = if hashed {
            iter::repeat_with(|| Some(buffer.read_n_bytes(64))).take(entry_count).collect()
        } else {
            vec![None; entry_count]
        };

        let (compressed_sizes, sizes): (Vec<_>, Vec<_>) = if unk4 {
            iter::repeat_with(|| (Some(buffer.read_unsigned_int()), Some(buffer.read_unsigned_int())))
                .take(entry_count)
                .unzip()
        } else {
            (vec![None; entry_count], vec![None; entry_count])
        };

        let versions = iter::repeat_with(|| buffer.read_int()).take(entry_count).collect::<Vec<i32>>();

        let child_counts = iter::repeat_with(|| {
            Ok({
                if format >= 7 {
                    buffer.read_smart32()?
                } else {
                    buffer.read_unsigned_short() as u32
                }
            })
        })
        .take(entry_count)
        .collect::<CacheResult<Vec<u32>>>()?;

        let child_indices = child_counts
            .iter()
            .map(|count| {
                Ok({
                    iter::repeat_with(|| {
                        Ok({
                            if format >= 7 {
                                buffer.read_smart32()?
                            } else {
                                buffer.read_unsigned_short() as u32
                            }
                        })
                    })
                    .take(*count as usize)
                    .collect::<CacheResult<Vec<u32>>>()?
                    .into_iter()
                    .accumulate(Add::add)
                    .collect::<Vec<u32>>()
                })
            })
            .collect::<CacheResult<Vec<Vec<u32>>>>()?;

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
        .collect::<HashMap<_, _>>();

        Ok(Self { metadatas })
    }

    /// View a specific [`Metadata`] of `self`.
    #[inline(always)]
    pub fn get(&self, archive_id: &u32) -> Option<&Metadata> {
        self.metadatas.get(&archive_id)
    }

    /// An iterator visiting all key-value pairs in arbitrary order. The iterator element type is `(&'a u32, &'a Metadata)`.
    #[inline(always)]
    pub fn iter(&self) -> hash_map::Iter<'_, u32, Metadata> {
        self.metadatas.iter()
    }
}

impl IntoIterator for IndexMetadata {
    type Item = (u32, Metadata);

    type IntoIter = hash_map::IntoIter<u32, Metadata>;

    fn into_iter(self) -> Self::IntoIter {
        self.metadatas.into_iter()
    }
}

#[pyproto]
impl PyObjectProtocol for Metadata {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Metadata({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Metadata({})", serde_json::to_string(self).unwrap()))
    }
}
