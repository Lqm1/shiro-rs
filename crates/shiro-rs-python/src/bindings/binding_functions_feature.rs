use super::*;
#[pyfunction(name = "feature_frame_count")]
#[pyo3(signature = (bytes, dimensions))]
pub fn binding_feature_frame_count(bytes: u64, dimensions: usize) -> pyo3::PyResult<usize> {
    crate::api::phones::feature_frame_count(bytes, dimensions)
}
