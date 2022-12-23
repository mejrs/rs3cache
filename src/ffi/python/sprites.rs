use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use pyo3::{
    exceptions::{PyKeyError, PyReferenceError},
    prelude::*,
    types::PyBytes,
};
use rs3cache_backend::index::CachePath;

use crate::{
    cache::{
        arc::Archive,
        index::{self, CacheIndex, Initial},
        meta::IndexMetadata,
    },
    cli::Config,
    definitions::{indextype::IndexType, sprites},
    ffi::python::PyIndexMetadata,
};
/// Obtained from [`PySprites`]'s [`get`](PySprites::get) method.
#[pyclass(name = "Sprite")]
pub struct PySprite {
    #[pyo3(get)]
    archive_id: u32,
    #[pyo3(get)]
    frames: BTreeMap<usize, PyFrame>,
}

#[pymethods]
impl PySprite {
    fn get(&self, id: usize) -> PyResult<PyFrame> {
        self.frames.get(&id).ok_or_else(|| PyKeyError::new_err("Key not in sprite")).cloned()
    }

    fn __getitem__(&self, id: usize) -> PyResult<PyFrame> {
        self.get(id)
    }
}

#[pyclass(name = "Frame")]
#[derive(Clone, Debug)]
pub struct PyFrame {
    constructor: Py<PyAny>,
    #[pyo3(get)]
    id: usize,
    im: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
}

#[pymethods]
impl PyFrame {
    fn image(&self, py: Python) -> PyResult<Py<PyAny>> {
        let image = self.constructor.call1(py, ("RGBA", self.im.dimensions(), self.bytes(py)))?;
        Ok(image)
    }

    fn bytes<'py>(&self, py: Python<'py>) -> &'py PyBytes {
        PyBytes::new(py, self.im.as_raw())
    }

    fn dimensions(&self) -> (u32, u32) {
        self.im.dimensions()
    }
}

#[pyclass(name = "Sprites")]
pub struct PySprites {
    constructor: Py<PyAny>,
    inner: Option<CacheIndex<Initial>>,
}

#[pymethods]
impl PySprites {
    /// Constructor of [`Sprites`].
    #[new]
    #[args(path = "None")]
    fn __new__(py: Python, path: Option<PathBuf>) -> PyResult<Self> {
        let mut config = Config::env();
        if let Some(path) = path {
            config.input = Arc::new(CachePath::Given(path))
        }
        let constructor = py.import("PIL")?.getattr("Image")?.getattr("frombytes")?.into();

        Ok(Self {
            constructor,
            inner: Some(CacheIndex::new(IndexType::SPRITES, config.input)?),
        })
    }

    fn get(&self, py: Python, id: u32) -> PyResult<PySprite> {
        let archive = self.archive(id)?;
        let sprite = archive.file(&0).and_then(sprites::deserialize);
        let sprite = sprite.map(|frames| PySprite {
            archive_id: archive.archive_id(),
            frames: frames
                .into_iter()
                .map(|(id, im)| {
                    (
                        id,
                        PyFrame {
                            constructor: self.constructor.clone_ref(py),
                            id,
                            im,
                        },
                    )
                })
                .collect(),
        })?;
        Ok(sprite)
    }

    fn __getitem__(&self, py: Python, id: u32) -> PyResult<PySprite> {
        self.get(py, id).map_err(|_| PyKeyError::new_err("key not in sprites"))
    }

    /// Get a specific [`Archive`].
    ///
    /// # Exceptions
    ///
    /// Raises `ValueError` if the archive cannot be found.
    pub fn archive(&self, archive_id: u32) -> PyResult<Archive> {
        Ok(self
            .inner
            .as_ref()
            .ok_or_else(|| PyReferenceError::new_err("PySprites is not available after using `iter()`"))?
            .archive(archive_id)?)
    }

    /// Returns the [`Metadata`] of all archives in `self`.
    pub fn metadatas(&self) -> PyResult<PyIndexMetadata> {
        let meta: IndexMetadata = self
            .inner
            .as_ref()
            .ok_or_else(|| PyReferenceError::new_err("PySprites is not available after using `iter()`"))?
            .metadatas()
            .clone();

        Ok(PyIndexMetadata { inner: Some(meta) })
    }

    fn __iter__(&mut self, py: Python) -> PyResult<Py<PySpritesIter>> {
        let inner = std::mem::take(&mut self.inner);
        let inner = inner
            .ok_or_else(|| PyReferenceError::new_err("PySprites is not available after using `iter()`"))?
            .into_iter();

        let iter = PySpritesIter {
            constructor: self.constructor.clone_ref(py),
            inner,
        };
        Py::new(py, iter)
    }
}

/// Iterator over all archives in an Index.
#[pyclass(name = "SpritesIter")]
pub struct PySpritesIter {
    constructor: Py<PyAny>,
    inner: index::IntoIter,
}

#[pymethods]
impl PySpritesIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<PySprite> {
        // no archive currently expose can fail here once fully loaded
        self.inner.next().map(Result::unwrap).map(|archive| {
            archive
                .file(&0)
                .and_then(sprites::deserialize)
                .map(|frames| PySprite {
                    archive_id: archive.archive_id(),
                    frames: frames
                        .into_iter()
                        .map(|(id, im)| {
                            (
                                id,
                                PyFrame {
                                    constructor: self.constructor.clone_ref(py),
                                    id,
                                    im,
                                },
                            )
                        })
                        .collect(),
                })
                .unwrap()
        })
    }
}
