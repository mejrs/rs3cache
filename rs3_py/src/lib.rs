use pyo3::prelude::*;

#[pymodule]
fn rs3(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    rs3cache::ffi::python::initializer(py, m)?;
    Ok(())
}
