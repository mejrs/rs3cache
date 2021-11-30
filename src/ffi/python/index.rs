use std::{collections::btree_map, path::PathBuf};

use pyo3::{exceptions::PyReferenceError, prelude::*, PyIterProtocol, PyObjectProtocol};

use crate::{
    cache::{
        arc::Archive,
        index::{self, CacheIndex, Initial},
        meta::{IndexMetadata, Metadata},
    },
    cli::Config,
};

/// Wrapper over [`CacheIndex`]. The Python alias for this is `Index`
///
/// # Examples
/// ```python
/// from rs3cache import Index
///
/// index = Index(2)
///```
/// # Exceptions
/// Raises `FileNotFoundError` if the cache cannot be found.
#[pyclass(name = "Index")]
pub struct PyCacheIndex {
    inner: Option<CacheIndex<Initial>>,
}

#[pymethods]
impl PyCacheIndex {
    /// Constructor of [`PyCacheIndex`].
    #[new]
    #[args(index_id, path = "None")]
    fn new(index_id: u32, path: Option<PathBuf>) -> PyResult<Self> {
        let mut config = Config::env();
        if let Some(path) = path {
            config.input = path
        }

        Ok(Self {
            inner: Some(CacheIndex::new(index_id, &config.input)?),
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
}

#[pyproto]
impl PyObjectProtocol for PyCacheIndex {
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
}

#[pyproto]
impl PyIterProtocol for PyCacheIndex {
    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<PyCacheIndexIter>> {
        let inner = std::mem::take(&mut (*slf).inner);
        let inner = inner
            .ok_or_else(|| PyReferenceError::new_err("CacheIndex is not available after using `iter()`"))?
            .into_iter();

        let iter = PyCacheIndexIter { inner };
        Py::new(slf.py(), iter)
    }
}

/// Iterator over all archives in an Index.
#[pyclass(name = "IndexIter")]
pub struct PyCacheIndexIter {
    inner: index::IntoIter,
}

#[pyproto]
impl PyIterProtocol for PyCacheIndexIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<Archive> {
        // no archive currently expose can fail here once fully loaded
        slf.inner.next().map(Result::unwrap)
    }
}

#[pyclass(name = "IndexMetadata")]
pub struct PyIndexMetadata {
    pub(crate) inner: Option<IndexMetadata>,
}

#[pyproto]
impl PyObjectProtocol for PyIndexMetadata {
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
}

#[pyproto]
impl PyIterProtocol for PyIndexMetadata {
    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<PyIndexMetadataIter>> {
        let inner = std::mem::take(&mut (*slf).inner);
        let inner = inner
            .ok_or_else(|| PyReferenceError::new_err("IndexMetadata is not available after using `iter()`"))?
            .into_iter();

        let iter = PyIndexMetadataIter { inner };
        Py::new(slf.py(), iter)
    }
}

/// Iterator over all archives in an Index.
#[pyclass(name = "IndexMetadataIter")]
pub struct PyIndexMetadataIter {
    inner: btree_map::IntoIter<u32, Metadata>,
}

#[pyproto]
impl PyIterProtocol for PyIndexMetadataIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<(u32, Metadata)> {
        slf.inner.next()
    }
}
