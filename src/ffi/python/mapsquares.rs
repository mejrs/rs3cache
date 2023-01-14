use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use pyo3::{
    exceptions::{PyIndexError, PyReferenceError, PyRuntimeError, PyTypeError},
    prelude::*,
    types::{PyInt, PyList},
};
use rs3cache_backend::index::CachePath;

use crate::{
    cli::Config,
    definitions::{
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
            config.input = Arc::new(CachePath::Given(path))
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
            .map_err(|_| PyIndexError::new_err(format!("i was {i}. It must satisfy 0 <= i <= 100.")))?;
        let j = j
            .downcast::<PyInt>()
            .map_err(|_| PyTypeError::new_err(format!("j was of type {}. j must be an integer.", j.get_type())))?
            .extract::<u8>()
            .map_err(|_| PyIndexError::new_err(format!("j was {j}. It must satisfy 0 <= j <= 200.")))?;

        if i >= 100 {
            Err(PyIndexError::new_err(format!("i was {i}. It must satisfy 0 <= i <= 100.")))
        } else if j >= 200 {
            Err(PyIndexError::new_err(format!("j was {j}. It must satisfy 0 <= j <= 200.")))
        } else {
            let sq = self
                .mapsquares
                .as_ref()
                .ok_or_else(|| PyReferenceError::new_err("Mapsquares is not available after using `iter()`"))?
                .get(i, j)?;

            Ok(PyMapSquare { inner: sq })
        }
    }

    fn __iter__(&mut self, py: Python) -> PyResult<Py<PyMapSquaresIter>> {
        let inner = std::mem::take(&mut self.mapsquares);
        let inner = inner.ok_or_else(|| PyReferenceError::new_err("Mapsquares is not available after using `iter()`"))?;
        let inner = inner.into_iter();

        let iter = PyMapSquaresIter { inner };
        Py::new(py, iter)
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

    fn __next__(&mut self, py: Python) -> Option<PyMapSquare> {
        match self.inner.next() {
            Some(Ok(sq)) => Some(PyMapSquare { inner: sq }),
            Some(Err(e)) => {
                PyRuntimeError::new_err(format!("Error: {e}")).restore(py);
                None
            }
            None => None,
        }
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
    pub fn locations<'gil>(&self, py: Python<'gil>) -> PyResult<&'gil PyList> {
        Ok(PyList::new(py, self.inner.locations()?.iter().copied()))
    }

    /// The water [`Location`]s in a mapsquare.
    #[cfg(feature = "rs3")]
    pub fn water_locations<'gil>(&self, py: Python<'gil>) -> PyResult<&'gil PyList> {
        Ok(PyList::new(py, self.inner.water_locations()?.iter().copied()))
    }

    /// The [`Tile`]s in a mapsquare.   
    pub fn tiles(&self) -> PyResult<BTreeMap<(u8, u8, u8), Tile>> {
        let tiles = self.inner.tiles()?;
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
