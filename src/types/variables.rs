#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;

/// A bitmapping of a [`Varp`]
#[cfg_attr(feature = "pyo3", pyclass(frozen))]
#[derive(Serialize, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Varbit {
    ///The value of the `Varbit`. Cannot be `Some(u16::MAX)`.
    pub val: Option<u16>,
}

impl Varbit {
    /// Constructor for [`Varbit`]
    pub fn new(id: u16) -> Self {
        if id != u16::MAX {
            Self { val: Some(id) }
        } else {
            Self { val: None }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Varbit {
    #[classattr]
    fn r#type() -> String {
        "varbit".to_string()
    }

    #[getter]
    fn val(&self) -> PyResult<Option<u16>> {
        Ok(self.val)
    }
}

/// A player variable
#[cfg_attr(feature = "pyo3", pyclass(frozen))]
#[derive(Serialize, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Varp {
    ///The value of the `Varp`. Cannot be `Some(u16::MAX)`.
    pub val: Option<u16>,
}

impl Varp {
    /// Constructor for [`Varp`].
    pub fn new(id: u16) -> Self {
        if id != u16::MAX {
            Self { val: Some(id) }
        } else {
            Self { val: None }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Varp {
    #[classattr]
    fn r#type() -> String {
        "varp".to_string()
    }

    #[getter]
    fn val(&self) -> PyResult<Option<u16>> {
        Ok(self.val)
    }
}

/// A variable containing either a Varp or Varbit.
#[derive(Serialize, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum VarpOrVarbit {
    /// See [`Varbit`].
    #[serde(rename = "varbit")]
    Varbit(u16),
    /// See [`Varp`].
    #[serde(rename = "varp")]
    Varp(u16),
}

impl VarpOrVarbit {
    /// Constructor for VarpOrVarbit.
    ///
    /// # Panics
    /// Panics if `varp` and `varbit` have the same discriminant, i.e. one has to be `Some` and the other has to be `None`.
    pub fn new(varp: Varp, varbit: Varbit) -> Self {
        match (varp.val, varbit.val) {
            (Some(id), None) => Self::Varp(id),
            (None, Some(id)) => Self::Varbit(id),
            other => panic!("Invalid variable pattern {other:?}."),
        }
    }
}

#[cfg(feature = "pyo3")]
impl IntoPy<PyObject> for VarpOrVarbit {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Self::Varbit(id) => Varbit { val: Some(id) }.into_py(py),
            Self::Varp(id) => Varp { val: Some(id) }.into_py(py),
        }
    }
}
