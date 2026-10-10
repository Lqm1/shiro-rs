use super::*;
#[pyfunction(name = "rawfloat_read")]
#[pyo3(signature = (bytes, maximum_samples))]
pub fn binding_rawfloat_read(bytes: Vec<u8>, maximum_samples: usize) -> pyo3::PyResult<Vec<f32>> {
    crate::api::rawfloat::rawfloat_read(&bytes, maximum_samples)
}
#[pyfunction(name = "rawfloat_write")]
#[pyo3(signature = (values))]
pub fn binding_rawfloat_write(values: Vec<f32>) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyBytes>> {
    (crate::api::rawfloat::rawfloat_write(&values)).map(|value| {
        let bytes = value;
        pyo3::Python::attach(|py| pyo3::types::PyBytes::new(py, &bytes).unbind())
    })
}
