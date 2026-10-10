use super::*;
#[pyfunction(name = "label_output_path")]
#[pyo3(signature = (filename, suffix))]
pub fn binding_label_output_path(filename: &str, suffix: &str) -> pyo3::PyResult<String> {
    Ok(crate::api::labels::label_output_path(filename, suffix))
}
