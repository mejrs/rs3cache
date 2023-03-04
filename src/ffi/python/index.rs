use std::{
    collections::{btree_map, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::Arc,
};

use pyo3::{
    class::basic::CompareOp,
    exceptions::{PyIndexError, PyReferenceError},
    prelude::*,
};
use rs3cache_backend::{
    arc::Archive,
    index::{self, CacheIndex, CachePath, Initial},
    meta::{IndexMetadata, Metadata},
};

use crate::cli::Config;

/// Wrapper over [`CacheIndex`]. The Python alias for this is `Index`
///
/// # Examples
/// ```python
/// from rs3cache import Index
///
/// index = Index(2)
///```
/// # Exceptions
/// Raises `FileMissingError` if the cache cannot be found.
#[pyclass(name = "Index")]
pub struct PyCacheIndex {
    inner: Option<CacheIndex<Initial>>,
}

#[pymethods]
impl PyCacheIndex {
    /// Constructor of [`PyCacheIndex`].
    #[new]
    #[pyo3(signature=(index_id, path=None))]
    fn new(index_id: u32, path: Option<PathBuf>) -> PyResult<Self> {
        let mut config = Config::env();
        if let Some(path) = path {
            config.input = Arc::new(CachePath::Given(path))
        }

        Ok(Self {
            inner: Some(CacheIndex::new(index_id, config.input)?),
        })
    }

    /// Get a specific [`Archive`].
    /// # Exceptions
    /// Raises `ValueError` if the archive cannot be found.
    pub fn archive(&self, archive_id: u32) -> PyResult<Archive> {
        Ok(self
            .inner
            .as_ref()
            .ok_or_else(|| PyReferenceError::new_err("CacheIndex is not available after using `iter()`"))?
            .archive(archive_id)?)
    }

    /// Returns the [`Metadata`] of all archives in `self`.
    pub fn metadatas(&self) -> PyResult<PyIndexMetadata> {
        let meta: IndexMetadata = self
            .inner
            .as_ref()
            .ok_or_else(|| PyReferenceError::new_err("CacheIndex is not available after using `iter()`"))?
            .metadatas()
            .clone();

        Ok(PyIndexMetadata { inner: Some(meta) })
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Index({})",
            self.inner
                .as_ref()
                .ok_or_else(|| PyReferenceError::new_err("CacheIndex is not available after using `iter()`"))?
                .index_id()
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "Index({})",
            self.inner
                .as_ref()
                .ok_or_else(|| PyReferenceError::new_err("CacheIndex is not available after using `iter()`"))?
                .index_id()
        ))
    }

    fn __iter__(&mut self, py: Python) -> PyResult<Py<PyCacheIndexIter>> {
        let inner = std::mem::take(&mut self.inner);
        let inner = inner
            .ok_or_else(|| PyReferenceError::new_err("CacheIndex is not available after using `iter()`"))?
            .into_iter();

        let iter = PyCacheIndexIter { inner };
        Py::new(py, iter)
    }
}

/// Iterator over all archives in an Index.
#[pyclass(name = "IndexIter")]
pub struct PyCacheIndexIter {
    inner: index::IntoIter,
}

#[pymethods]
impl PyCacheIndexIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self) -> Option<Archive> {
        // no archive currently expose can fail here once fully loaded
        self.inner.next().map(Result::unwrap)
    }
}

#[pyclass(name = "IndexMetadata")]
pub struct PyIndexMetadata {
    pub(crate) inner: Option<IndexMetadata>,
}

#[pymethods]
impl PyIndexMetadata {
    fn __getitem__(&self, index: u32) -> PyResult<Metadata> {
        let meta = self
            .inner
            .as_ref()
            .ok_or_else(|| PyReferenceError::new_err("IndexMetadata is not available after using `iter()`"))?
            .metadatas()
            .get(&index)
            .ok_or_else(|| PyIndexError::new_err("Key not present"))?
            .to_owned();

        Ok(meta)
    }
    fn __repr__(&self) -> PyResult<String> {
        let inner = self
            .inner
            .as_ref()
            .ok_or_else(|| PyReferenceError::new_err("IndexMetadata is not available after using `iter()`"))?
            .metadatas();
        Ok(format!("IndexMetadata({})", serde_json::to_string(inner).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        let inner = self
            .inner
            .as_ref()
            .ok_or_else(|| PyReferenceError::new_err("IndexMetadata is not available after using `iter()`"))?
            .metadatas();
        Ok(format!("IndexMetadata({})", serde_json::to_string(inner).unwrap()))
    }

    fn __iter__(&mut self, py: Python) -> PyResult<Py<PyIndexMetadataIter>> {
        let inner = std::mem::take(&mut self.inner);
        let inner = inner
            .ok_or_else(|| PyReferenceError::new_err("IndexMetadata is not available after using `iter()`"))?
            .into_iter();

        let iter = PyIndexMetadataIter { inner };
        Py::new(py, iter)
    }

    fn __hash__(&self) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn __richcmp__(&self, other: &PyIndexMetadata, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => Ok(self.inner < other.inner),
            CompareOp::Le => Ok(self.inner <= other.inner),
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            CompareOp::Gt => Ok(self.inner > other.inner),
            CompareOp::Ge => Ok(self.inner >= other.inner),
        }
    }
}

/// Iterator over all archives in an Index.
#[pyclass(name = "IndexMetadataIter")]
pub struct PyIndexMetadataIter {
    inner: btree_map::IntoIter<u32, Metadata>,
}

#[pymethods]
impl PyIndexMetadataIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self) -> Option<(u32, Metadata)> {
        self.inner.next()
    }
}
