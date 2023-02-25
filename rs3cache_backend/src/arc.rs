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

use ::error::Context;
use bytes::{Buf, Bytes};
use itertools::izip;
#[cfg(feature = "pyo3")]
use pyo3::{exceptions::PyKeyError, prelude::*, types::PyBytes};

use crate::{
    buf::BufExtra,
    error::{self, CacheError, CacheResult},
    meta::Metadata,
};

/// A collection of files.
#[cfg_attr(feature = "pyo3", pyclass(frozen))]
#[derive(Clone, Default)]
pub struct Archive {
    pub(crate) index_id: u32,
    pub(crate) archive_id: u32,
    pub(crate) files: BTreeMap<u32, Bytes>,
    #[cfg(feature = "dat")]
    pub(crate) files_named: BTreeMap<i32, Bytes>,
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

    #[cfg(any(feature = "sqlite", feature = "dat2"))]
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

            #[cfg(feature = "sqlite")]
            child_count => {
                use crate::utils::adapters::Pairwisor;

                assert_eq!(data[0], 1);

                let mut offset_data = data.slice(1..((child_count + 1) * 4 + 1) as usize);

                let offsets = std::iter::repeat_with(|| offset_data.get_i32() as usize)
                    .take(child_count as usize + 1)
                    .pairwise();

                izip!(metadata.child_indices(), offsets)
                    .map(|(i, (start, end))| (*i, data.slice(start..end)))
                    .collect::<BTreeMap<_, _>>()
            }

            #[cfg(feature = "dat2")]
            child_count => {
                use crate::utils::adapters::Accumulator;
                let mut data = data;

                let n_chunks = *data.last().unwrap() as usize;

                let offset_start = data.len().checked_sub(4 * n_chunks * (child_count as usize) + 1).unwrap();
                let mut offset_data = data.split_off(offset_start);

                let offsets = std::iter::repeat_with(|| offset_data.get_i32())
                    .take(child_count as usize)
                    .accumulate(|x, y| x + y);

                izip!(metadata.child_indices(), offsets)
                    .map(|(i, n)| (*i, data.split_to(n.try_into().unwrap())))
                    .collect::<BTreeMap<_, _>>()
            }
        };

        Archive { index_id, archive_id, files }
    }

    /// Gets a File.
    pub fn file(&self, file_id: &u32) -> Option<Bytes> {
        self.files.get(file_id).cloned()
    }

    /// Take the files. Consumes the [`Archive`].
    pub fn take_files(self) -> BTreeMap<u32, Bytes> {
        self.files
    }

    /// The quantity of files currently in the archive.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    #[cfg(feature = "dat")]
    pub(crate) fn deserialize_jag(metadata: &Metadata, mut buffer: Bytes) -> CacheResult<Archive> {
        use std::io::Read;

        #[derive(Debug)]
        struct JagHeader {
            filename: i32,
            decompressed_len: u32,
            compressed_len: u32,
        }

        assert_eq!(metadata.index_id(), 0, "called deserialize_jag on data not from index 0");

        let decompressed_len = buffer.try_get_uint(3).context(error::Read)?;
        let compressed_len = buffer.try_get_uint(3).context(error::Read)?;

        let extracted = if decompressed_len != compressed_len {
            let mut compressed = bytes::BytesMut::from(b"BZh1".as_slice());
            compressed.extend(buffer.split_to(compressed_len as usize));

            let mut decoded = Vec::with_capacity(decompressed_len as usize);

            let mut decoder = bzip2_rs::DecoderReader::new(&compressed[..]);

            decoder.read_to_end(&mut decoded).unwrap();
            buffer = Bytes::from(decoded);
            true
        } else {
            false
        };

        let files_length = buffer.get_u16();

        let mut headers = buffer.split_to((files_length * 10).try_into().unwrap());
        let mut archive = Archive {
            index_id: metadata.index_id(),
            archive_id: metadata.archive_id(),
            ..Default::default()
        };

        for i in 0..files_length {
            let header = JagHeader {
                filename: headers.try_get_i32().context(error::Read)?,
                decompressed_len: headers.try_get_uint(3).context(error::Read)? as u32,
                compressed_len: headers.try_get_uint(3).context(error::Read)? as u32,
            };

            let decompressed = if extracted {
                buffer.split_to(header.decompressed_len as usize)
            } else {
                let mut compressed = bytes::BytesMut::from(b"BZh1".as_slice());
                compressed.extend(buffer.split_to(header.compressed_len as usize));

                use pyo3::{prelude::*, types::PyBytes};
                pyo3::prepare_freethreaded_python();

                let res = Python::with_gil(|py| {
                    let zlib = PyModule::import(py, "bz2")?;
                    let decompress = zlib.getattr("decompress")?;
                    let bytes = PyBytes::new(py, &compressed);
                    let value = decompress.call1((bytes,))?;
                    value.extract::<Vec<u8>>()
                })
                .unwrap();
                Bytes::from(res)
            };
            archive.files.insert(i as u32, decompressed.clone());
            archive.files_named.insert(header.filename, decompressed);
        }
        assert_eq!(buffer.remaining(), 0);

        Ok(archive)
    }

    #[cfg(feature = "dat")]
    pub fn file_named(&self, name: impl AsRef<str>) -> CacheResult<Bytes> {
        let name = name.as_ref();

        let hash = crate::hash::hash_archive(name);

        self.files_named
            .get(&hash)
            .with_context(|| crate::index::FileMissingNamed {
                index_id: self.index_id,
                archive_id: self.archive_id,
                name: name.into(),
            })
            .context(error::Integrity)
            .cloned()
    }

    /// Take the files. Consumes the [`Archive`].
    #[cfg(feature = "dat")]
    pub fn take_files_named(self) -> BTreeMap<i32, Bytes> {
        self.files_named
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Archive {
    #[pyo3(name = "file")]
    fn py_file<'p>(&self, py: Python<'p>, file_id: u32) -> PyResult<&'p PyBytes> {
        if let Some(file) = self.files.get(&file_id) {
            Ok(PyBytes::new(py, file))
        } else {
            Err(PyKeyError::new_err(format!("File {file_id} is not present.")))
        }
    }

    #[pyo3(name = "files")]
    fn py_files<'p>(&self, py: Python<'p>) -> PyResult<BTreeMap<u32, &'p PyBytes>> {
        Ok(self.files.iter().map(|(&id, file)| (id, PyBytes::new(py, file))).collect())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Archive({}, {})", self.index_id(), self.archive_id()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Archive({}, {})", self.index_id(), self.archive_id()))
    }
}
