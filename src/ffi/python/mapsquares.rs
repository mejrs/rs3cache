use std::{collections::BTreeMap, path::PathBuf};

use fstrings::{f, format_args_f};
use pyo3::{
    exceptions::{PyIndexError, PyKeyError, PyReferenceError, PyTypeError},
    prelude::*,
    types::PyInt,
};

use crate::{
    cli::Config,
    definitions::{
        locations::Location,
        mapsquares::{MapSquare, MapSquareIterator, MapSquares},
        tiles::Tile,
    },
};

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
    mapsquares: Option<MapSquares>,
}

#[pymethods]
impl PyMapSquares {
    #[new]
    #[args(path = "None")]
    fn new(path: Option<PathBuf>) -> PyResult<Self> {
        let mut config = Config::env();
        if let Some(path) = path {
            config.input = path
        }
        Ok(Self {
            mapsquares: Some(MapSquares::new(&config)?),
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
        let i = i
            .downcast::<PyInt>()
            .map_err(|_| PyTypeError::new_err(format!("i was of type {}. i must be an integer.", i.get_type())))?
            .extract::<u8>()
            .map_err(|_| PyIndexError::new_err(format!("i was {}. It must satisfy 0 <= i <= 100.", i)))?;
        let j = j
            .downcast::<PyInt>()
            .map_err(|_| PyTypeError::new_err(format!("j was of type {}. j must be an integer.", j.get_type())))?
            .extract::<u8>()
            .map_err(|_| PyIndexError::new_err(format!("j was {}. It must satisfy 0 <= j <= 200.", j)))?;

        if i >= 100 {
            Err(PyIndexError::new_err(format!("i was {}. It must satisfy 0 <= i <= 100.", i)))
        } else if j >= 200 {
            Err(PyIndexError::new_err(format!("j was {}. It must satisfy 0 <= j <= 200.", j)))
        } else {
            let sq = self
                .mapsquares
                .as_ref()
                .ok_or_else(|| PyReferenceError::new_err("Mapsquares is not available after using `iter()`"))?
                .get(i, j)
                .ok_or_else(|| PyKeyError::new_err(f!("Mapsquare {i}, {j} is not present.")))?;

            Ok(PyMapSquare { inner: sq })
        }
    }

    fn __iter__(mut slf: PyRefMut<Self>) -> PyResult<Py<PyMapSquaresIter>> {
        let inner = std::mem::take(&mut (*slf).mapsquares);
        let inner = inner.ok_or_else(|| PyReferenceError::new_err("Mapsquares is not available after using `iter()`"))?;
        let inner = inner.into_iter();

        let iter = PyMapSquaresIter { inner };
        Py::new(slf.py(), iter)
    }
}

/// Iterator over all archives in an Index.
#[pyclass(name = "MapSquaresIter")]
pub struct PyMapSquaresIter {
    inner: MapSquareIterator,
}

#[pymethods]
impl PyMapSquaresIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyMapSquare> {
        (*slf).inner.next().map(|sq| PyMapSquare { inner: sq })
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
        self.inner.i()
    }

    /// The vertical [`MapSquare`] coordinate.
    ///
    /// It can have any value in the range `0..=200`.
    #[getter]
    pub fn j(&self) -> u8 {
        self.inner.j()
    }

    /// The [`Location`]s in a mapsquare.
    pub fn locations(&self) -> PyResult<Vec<Location>> {
        let locs = self.inner.get_locations()?.clone();
        Ok(locs)
    }

    /// The water [`Location`]s in a mapsquare.
    #[cfg(feature = "rs3")]
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

    fn __repr__(&self) -> String {
        format!("MapSquare({},{})", self.inner.i(), self.inner.j())
    }

    fn __str__(&self) -> String {
        format!("MapSquare({},{})", self.inner.i(), self.inner.j())
    }
}