//! Python bindings for `rs3cache`.
//!
//! # Setup
//!
//! You need to build binary wheels locally. The following instructions are only guaranteed on **windows**.
//! If you run into issues on other platforms, please follow [here](https://github.com/PyO3/setuptools-rust#binary-wheels-on-linux) or try [maturin](https://pypi.org/project/maturin/) instead.
//!
//! - `git clone https://github.com/mejrs/rs3cache`.
//! - Install [Python](https://www.python.org/downloads/), version 3.6 or newer.
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
        index::{CacheIndex, Initial},
        indextype::IndexType,
        meta::Metadata,
    },
    definitions::{location_configs::LocationConfig, locations::Location, mapsquares::MapSquare, npc_configs::NpcConfig, tiles::Tile},
};
use pyo3::{prelude::*, wrap_pyfunction, PyObjectProtocol};
use std::collections::HashMap;

#[pymodule]
fn rs3cache(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_location_definitions, m)?)?;
    m.add_function(wrap_pyfunction!(get_npc_definitions, m)?)?;
    m.add_class::<PyMapSquares>()?;
    Ok(())
}

/// Wrapper for [`LocationConfig::dump_all`]
#[pyfunction]
pub fn get_location_definitions() -> PyResult<HashMap<u32, LocationConfig>> {
    Ok(LocationConfig::dump_all()?)
}

/// Wrapper for [`NpcConfig::dump_all`]
#[pyfunction]
pub fn get_npc_definitions() -> PyResult<HashMap<u32, NpcConfig>> {
    Ok(NpcConfig::dump_all()?)
}

/// Container of [`PyMapSquare`]s.
/// Accessible with `from rs3cache import MapSquares`.
#[pyclass(name = "MapSquares")]
pub struct PyMapSquares {
    index: CacheIndex<Initial>,
}

#[pymethods]
impl PyMapSquares {
    /// Constructor for MapSquares:
    /// ```python
    /// from rs3cache import *
    ///
    /// mapsquares = MapSquares()
    ///```
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(Self {
            index: CacheIndex::new(IndexType::MAPSV2)?,
        })
    }
    /// Get a specific :
    /// ```python
    /// from rs3cache import *
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

///
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
