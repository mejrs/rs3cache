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
//! - Install the [Rust compiler](https://rustup.rs/).
//! - Configure rustup to use the nightly version: `rustup default nightly`.
//! - Navigate to this repository and run `python setup.py install`.
//! - Either:
//!     - Create a system variable named `RUNESCAPE_CACHE_FOLDER` and set its value to where your cache is located.
//!       Typically, this is `%ProgramData%\Jagex\RuneScape`.
//!     - Copy the entire cache and place it in the `raw` folder.
//!
//!  # Usage
//!
//! All the following can be imported as:
//! ```python
//! from rs3cache import *
//! ```
//!
//!
//!
//!
use crate::{
    cache::{
        arc::Archive,
        index::{self, CacheIndex, Initial},
        indextype::IndexType,
        meta::Metadata,
    },
    definitions::{
        location_configs::LocationConfig, locations::Location, mapsquares::MapSquare, npc_configs::NpcConfig, item_configs::ItemConfig,tiles::Tile,
        varbit_configs::VarbitConfig,
    },
};
use pyo3::{prelude::*, wrap_pyfunction, PyIterProtocol, PyObjectProtocol};
use std::collections::{BTreeMap, HashMap};

#[pymodule]
fn rs3cache(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_location_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_npc_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_item_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_varbit_configs, m)?)?;
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
    /// Raises `OverflowError` if `i` or `j` is not between 0 and 255.
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
    pub fn get(&self, i: u8, j: u8) -> PyResult<PyMapSquare> {
        let archive_id = (i as u32) | (j as u32) << 7;
        let archive = self.index.archive(archive_id)?;
        let sq = MapSquare::from_archive(archive);
        let metadata = self.index.metadatas().get(&archive_id).cloned();
        Ok(PyMapSquare { inner: sq, metadata })
    }
}

/// Obtained from [`PyMapSquares`]'s [`get`](PyMapSquares::get) method.
#[pyclass(name = "MapSquares")]
pub struct PyMapSquare {
    inner: MapSquare,
    #[pyo3(get)]
    metadata: Option<Metadata>,
}

#[pymethods]
impl PyMapSquare {
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
    pub fn tiles(&self) -> PyResult<HashMap<(u8, u8, u8), Tile>> {
        let tiles = self.inner.get_tiles()?;
        let map: HashMap<(u8, u8, u8), Tile> = tiles.indexed_iter().map(|((p, x, y), &t)| ((p as u8, x as u8, y as u8), t)).collect();
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
    pub fn metadatas(&self) -> PyResult<HashMap<u32, Metadata>>{
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
