//! Python bindings for `rs3cache`.
//!
//! # Setup
//!
//! You need to build binary wheels locally. The following instructions are only guaranteed on **windows**.
//! If you run into issues on other platforms, please follow [here](https://github.com/PyO3/setuptools-rust#binary-wheels-on-linux) or try [maturin](https://pypi.org/project/maturin/) instead.
//!
//! - `git clone https://github.com/mejrs/rs3cache`.
//! - Install [Python](https://www.python.org/downloads/ "Download Python"), version 3.9 (lower versions may work).
//!     - Check that pip is installed (`python -m pip --version`).
//!     - Install setuptools: `pip install setuptools`.
//!     - Install setuptools-rust: `pip install setuptools-rust`.
//! - [Install the Rust compiler](https://doc.rust-lang.org/stable/book/ch01-01-installation.html "Installation - The Rust Programming Language").
//! - Configure rustup to use the nightly version: `rustup default nightly`.
//! - Navigate to this repository and run `python setup.py install`.
//! - Either:
//!     - Create a system variable named `RUNESCAPE_CACHE_FOLDER` and set its value to where your cache is located.
//!       Typically, this is `%ProgramData%\Jagex\RuneScape`.
//!     - Copy the entire cache and place it in the `raw` folder.
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
    use std::collections::{BTreeMap, HashMap};

    use pyo3::{exceptions::PyIndexError, prelude::*, types::PyInt, wrap_pyfunction, PyIterProtocol, PyObjectProtocol};

    use crate::{
        cache::{
            arc::Archive,
            index::{self, CacheIndex, Initial},
            indextype::IndexType,
            meta::Metadata,
        },
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
        Ok(LocationConfig::dump_all()?.into_iter().collect())
    }

    /// Wrapper for [`NpcConfig::dump_all`]
    #[pyfunction]
    pub fn get_npc_configs() -> PyResult<BTreeMap<u32, NpcConfig>> {
        Ok(NpcConfig::dump_all()?.into_iter().collect())
    }

    /// Wrapper for [`NpcConfig::dump_all`]
    #[pyfunction]
    pub fn get_item_configs() -> PyResult<BTreeMap<u32, ItemConfig>> {
        Ok(ItemConfig::dump_all()?.into_iter().collect())
    }

    /// Wrapper for [`Struct::dump_all`]
    #[pyfunction]
    pub fn get_struct_configs() -> PyResult<BTreeMap<u32, Struct>> {
        Ok(Struct::dump_all()?.into_iter().collect())
    }

    /// Wrapper for [`Struct::dump_all`]
    #[pyfunction]
    pub fn get_enum_configs() -> PyResult<BTreeMap<u32, Enum>> {
        Ok(Enum::dump_all()?.into_iter().collect())
    }

    /// Wrapper for [`VarbitConfig::dump_all`]
    #[pyfunction]
    pub fn get_varbit_configs() -> PyResult<BTreeMap<u32, VarbitConfig>> {
        Ok(VarbitConfig::dump_all()?.into_iter().collect())
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
        index: CacheIndex<Initial>,
    }

    #[pymethods]
    impl PyMapSquares {
        #[new]
        fn new() -> PyResult<Self> {
            Ok(Self {
                index: CacheIndex::new(IndexType::MAPSV2)?,
            })
        }

        /// Get a specific mapsquare.
        ///
        /// # Exceptions
        /// Raises `ValueError` if `i` or `j` is not between 0 and 200.
        ///
        /// Raises `ValueError` if there is not a mapsquare at `(i,j)`.
        ///
        /// # Example
        /// ```python
        /// from rs3cache import MapSquares
        ///
        /// mapsquares = MapSquares()
        /// lumbridge = mapsquares.get(50, 50)
        ///```
        pub fn get(&self, i: &PyInt, j: &PyInt) -> PyResult<PyMapSquare> {
            let rust_i = i
                .extract::<u32>()
                .map_err(|_| PyIndexError::new_err(format!("i was {}. It must satisfy 0 <= i <= 100.", i)))?;
            let rust_j = j
                .extract::<u32>()
                .map_err(|_| PyIndexError::new_err(format!("j was {}. It must satisfy 0 <= j <= 200.", j)))?;

            if rust_i >= 100 {
                Err(PyIndexError::new_err(format!("i was {}. It must satisfy 0 <= i <= 100.", i)))
            } else if rust_j >= 200 {
                Err(PyIndexError::new_err(format!("j was {}. It must satisfy 0 <= j <= 200.", j)))
            } else {
                let archive_id = rust_i | rust_j << 7;
                let archive = self.index.archive(archive_id)?;
                let sq = MapSquare::from_archive(archive);
                let metadata = self.index.metadatas().get(&archive_id).cloned();
                Ok(PyMapSquare { inner: sq, metadata })
            }
        }
    }

    #[pyproto]
    impl PyIterProtocol for PyMapSquares {
        fn __iter__(slf: PyRef<Self>) -> PyResult<Py<PyMapSquaresIter>> {
            let iter = PyMapSquaresIter {
                inner: (*slf).index.try_clone()?.into_iter(),
            };
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
            let metadata = (*slf).inner.metadatas().get(&archive_id).cloned();
            Some(PyMapSquare { inner: sq, metadata })
        }
    }

    /// Obtained from [`PyMapSquares`]'s [`get`](PyMapSquares::get) method.
    #[pyclass(name = "MapSquare")]
    pub struct PyMapSquare {
        inner: MapSquare,
        #[pyo3(get)]
        metadata: Option<Metadata>,
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
        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("MapSquare({},{})", self.inner.i, self.inner.j))
        }

        fn __str__(&self) -> PyResult<String> {
            Ok(format!("MapSquare({},{})", self.inner.i, self.inner.j))
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
        inner: CacheIndex<Initial>,
    }

    #[pymethods]
    impl PyCacheIndex {
        #[new]
        /// Constructor of [`PyCacheIndex`].
        fn new(index_id: u32) -> PyResult<Self> {
            Ok(Self {
                inner: CacheIndex::new(index_id)?,
            })
        }

        /// Get a specific [`Archive`].
        /// # Exceptions
        /// Raises `ValueError` if the archive cannot be found.
        pub fn archive(&self, archive_id: u32) -> PyResult<Archive> {
            Ok(self.inner.archive(archive_id)?)
        }

        /// Returns the [`Metadata`] of all archives in `self`.
        pub fn metadatas(&self) -> PyResult<HashMap<u32, Metadata>> {
            Ok(self.inner.metadatas().metadatas().clone())
        }
    }

    #[pyproto]
    impl PyObjectProtocol for PyCacheIndex {
        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("Index({})", self.inner.index_id()))
        }

        fn __str__(&self) -> PyResult<String> {
            Ok(format!("Index({})", self.inner.index_id()))
        }
    }

    #[pyproto]
    impl PyIterProtocol for PyCacheIndex {
        fn __iter__(slf: PyRef<Self>) -> PyResult<Py<PyCacheIndexIter>> {
            let iter = PyCacheIndexIter {
                inner: slf.inner.try_clone()?.into_iter(),
            };
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
}
