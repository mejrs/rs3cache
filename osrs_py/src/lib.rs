use pyo3::prelude::*;

#[pymodule]
fn osrs(py: Python, m: &PyModule) -> PyResult<()> {
    rs3cache::ffi::python::initializer(py, m)?;
    Ok(())
}
