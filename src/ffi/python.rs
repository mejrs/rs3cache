//! Python bindings for `rs3cache`.
//!
//! See the README for help with installing this.
//!
//!  # Usage
//!
//! Usage examples are available in the [/rs3cache](https://github.com/mejrs/rs3cache/tree/master/rs3cache "Examples") folder.
//!
//! All the following can be imported as:
//! ```python
//! from rs3cache import *
//! ```
//! ## Functions
//!
//! The following functions are available:
//! ```python
//! get_location_configs()
//! get_npc_configs()
//! get_item_configs()
//! get_varbit_configs()
//! get_struct_configs()
//! get_enum_configs()
//! ```
//! ## Classes
//!
//! The following classes are available:
//! ```python
//! MapSquares
//! CacheIndex
//! ```

#[allow(missing_docs)]
#[cfg(feature = "pyo3")]
pub mod python_impl {
    use std::collections::{btree_map, BTreeMap};

    use pyo3::{
        exceptions::{PyIndexError, PyReferenceError, PyTypeError},
        prelude::*,
        types::PyInt,
        wrap_pyfunction, PyIterProtocol, PyObjectProtocol,
    };

    use crate::{
        cache::{
            arc::Archive,
            index::{self, CacheIndex, Initial},
            indextype::IndexType,
            meta::{IndexMetadata, Metadata},
        },
        cli::Config,
        definitions::{
            enums::Enum, item_configs::ItemConfig, location_configs::LocationConfig, locations::Location, mapsquares::MapSquare,
            npc_configs::NpcConfig, structs::Struct, tiles::Tile, varbit_configs::VarbitConfig,
        },
    };

    #[pymodule]
    fn rs3cache(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(get_location_configs, m)?)?;
        m.add_function(wrap_pyfunction!(get_npc_configs, m)?)?;
        m.add_function(wrap_pyfunction!(get_item_configs, m)?)?;
        m.add_function(wrap_pyfunction!(get_varbit_configs, m)?)?;
        m.add_function(wrap_pyfunction!(get_struct_configs, m)?)?;
        m.add_function(wrap_pyfunction!(get_enum_configs, m)?)?;

        m.add_class::<PyMapSquares>()?;
        m.add_class::<PyCacheIndex>()?;
        Ok(())
    }

    /// Wrapper for [`LocationConfig::dump_all`]
    #[pyfunction]
    pub fn get_location_configs() -> PyResult<BTreeMap<u32, LocationConfig>> {
        Ok(LocationConfig::dump_all(&Config::env())?)
    }

    /// Wrapper for [`NpcConfig::dump_all`]
    #[pyfunction]
    pub fn get_npc_configs() -> PyResult<BTreeMap<u32, NpcConfig>> {
        Ok(NpcConfig::dump_all(&Config::env())?)
    }

    /// Wrapper for [`NpcConfig::dump_all`]
    #[pyfunction]
    pub fn get_item_configs() -> PyResult<BTreeMap<u32, ItemConfig>> {
        Ok(ItemConfig::dump_all(&Config::env())?)
    }

    /// Wrapper for [`Struct::dump_all`]
    #[pyfunction]
    pub fn get_struct_configs() -> PyResult<BTreeMap<u32, Struct>> {
        Ok(Struct::dump_all(&Config::env())?)
    }

    /// Wrapper for [`Struct::dump_all`]
    #[pyfunction]
    pub fn get_enum_configs() -> PyResult<BTreeMap<u32, Enum>> {
        Ok(Enum::dump_all(&Config::env())?)
    }

    /// Wrapper for [`VarbitConfig::dump_all`]
    #[pyfunction]
    pub fn get_varbit_configs() -> PyResult<BTreeMap<u32, VarbitConfig>> {
        Ok(VarbitConfig::dump_all(&Config::env())?)
    }

    /// Container of [`PyMapSquare`]s.
    /// Accessible with `from rs3cache import MapSquares`.
    /// # Example
    /// ```python
    /// from rs3cache import *
    ///
    /// mapsquares = MapSquares()
    ///```
    #[pyclass(name = "MapSquares")]
    pub struct PyMapSquares {
        index: Option<CacheIndex<Initial>>,
    }

    #[pymethods]
    impl PyMapSquares {
        #[new]
        fn new() -> PyResult<Self> {
            Ok(Self {
                index: Some(CacheIndex::new(IndexType::MAPSV2, &Config::env())?),
            })
        }

        /// Get a specific mapsquare.
        ///
        /// # Exceptions
        /// Raises `TypeError` if `i` and `j` are not integers.
        /// Raises `IndexError` if not(0 <= i <= 100 and 0 <= j <= 200)
        /// Raises `ValueError` if there is not a mapsquare at i, j.
        ///
        /// # Example
        /// ```python
        /// from rs3cache import MapSquares
        ///
        /// mapsquares = MapSquares()
        /// lumbridge = mapsquares.get(50, 50)
        ///```
        #[pyo3(text_signature = "($self, i, j)")]
        pub fn get(&self, i: &PyAny, j: &PyAny) -> PyResult<PyMapSquare> {
            let rust_i = i
                .downcast::<PyInt>()
                .map_err(|_| PyTypeError::new_err(format!("i was of type {}. i must be an integer.", i.get_type())))?
                .extract::<u32>()
                .map_err(|_| PyIndexError::new_err(format!("i was {}. It must satisfy 0 <= i <= 100.", i)))?;
            let rust_j = j
                .downcast::<PyInt>()
                .map_err(|_| PyTypeError::new_err(format!("j was of type {}. j must be an integer.", j.get_type())))?
                .extract::<u32>()
                .map_err(|_| PyIndexError::new_err(format!("j was {}. It must satisfy 0 <= j <= 200.", j)))?;

            if rust_i >= 100 {
                Err(PyIndexError::new_err(format!("i was {}. It must satisfy 0 <= i <= 100.", i)))
            } else if rust_j >= 200 {
                Err(PyIndexError::new_err(format!("j was {}. It must satisfy 0 <= j <= 200.", j)))
            } else {
                let archive_id = rust_i | rust_j << 7;
                let archive = self
                    .index
                    .as_ref()
                    .ok_or_else(|| PyReferenceError::new_err("Mapsquares is not available after using `iter()`"))?
                    .archive(archive_id)?;
                let sq = MapSquare::from_archive(archive);
                Ok(PyMapSquare { inner: sq })
            }
        }
    }

    #[pyproto]
    impl PyIterProtocol for PyMapSquares {
        fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<PyMapSquaresIter>> {
            let inner: Option<CacheIndex<Initial>> = std::mem::take(&mut (*slf).index);
            let inner: CacheIndex<Initial> = inner.ok_or_else(|| PyReferenceError::new_err("Mapsquares is not available after using `iter()`"))?;

            let iter = PyMapSquaresIter { inner: inner.into_iter() };
            Py::new(slf.py(), iter)
        }
    }

    /// Iterator over all archives in an Index.
    #[pyclass(name = "MapSquaresIter")]
    pub struct PyMapSquaresIter {
        inner: index::IntoIter,
    }

    #[pyproto]
    impl PyIterProtocol for PyMapSquaresIter {
        fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
            slf
        }

        fn __next__(mut slf: PyRefMut<Self>) -> Option<PyMapSquare> {
            let archive = (*slf).inner.next()?;
            let archive_id = archive.archive_id();
            let sq = MapSquare::from_archive(archive);
            Some(PyMapSquare { inner: sq })
        }
    }

    /// Obtained from [`PyMapSquares`]'s [`get`](PyMapSquares::get) method.
    #[pyclass(name = "MapSquare")]
    pub struct PyMapSquare {
        inner: MapSquare,
    }

    #[pymethods]
    impl PyMapSquare {
        /// The horizontal [`MapSquare`] coordinate.
        ///
        /// It can have any value in the range `0..=100`.
        #[getter]
        pub fn i(&self) -> u8 {
            self.inner.i
        }

        /// The vertical [`MapSquare`] coordinate.
        ///
        /// It can have any value in the range `0..=200`.
        #[getter]
        pub fn j(&self) -> u8 {
            self.inner.j
        }

        /// The [`Location`]s in a mapsquare.
        pub fn locations(&self) -> PyResult<Vec<Location>> {
            let locs = self.inner.get_locations()?.clone();
            Ok(locs)
        }

        /// The water [`Location`]s in a mapsquare.
        pub fn water_locations(&self) -> PyResult<Vec<Location>> {
            let locs = self.inner.get_water_locations()?.clone();
            Ok(locs)
        }

        /// The [`Tile`]s in a mapsquare.   
        pub fn tiles(&self) -> PyResult<BTreeMap<(u8, u8, u8), Tile>> {
            let tiles = self.inner.get_tiles()?;
            let map: BTreeMap<(u8, u8, u8), Tile> = tiles.indexed_iter().map(|((p, x, y), &t)| ((p as u8, x as u8, y as u8), t)).collect();
            Ok(map)
        }
    }

    #[pyproto]
    impl PyObjectProtocol for PyMapSquare {
        fn __repr__(&self) -> String {
            format!("MapSquare({},{})", self.inner.i, self.inner.j)
        }

        fn __str__(&self) -> String {
            format!("MapSquare({},{})", self.inner.i, self.inner.j)
        }
    }

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
        #[new]
        /// Constructor of [`PyCacheIndex`].
        fn new(index_id: u32) -> PyResult<Self> {
            Ok(Self {
                inner: Some(CacheIndex::new(index_id, &Config::env())?),
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
            slf.inner.next()
        }
    }

    #[pyclass(name = "IndexMetadata")]
    pub struct PyIndexMetadata {
        inner: Option<IndexMetadata>,
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
}
