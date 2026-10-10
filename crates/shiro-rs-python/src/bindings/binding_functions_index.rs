use super::*;
#[pyfunction(name = "index_append_suffix")]
#[pyo3(signature = (path, suffix))]
pub fn binding_index_append_suffix(path: &str, suffix: &str) -> pyo3::PyResult<String> {
    crate::api::index::index_append_suffix(path, suffix)
}
