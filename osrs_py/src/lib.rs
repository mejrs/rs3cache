use pyo3::prelude::*;

#[pymodule]
fn osrs(py: Python, m: &PyModule) -> PyResult<()> {
    osrscache::ffi::python::initializer(py, m)?;
    Ok(())
}
