use super::*;
#[pyfunction(name = "batch_feature_options")]
#[pyo3(signature = (value))]
pub fn binding_batch_feature_options(value: u32) -> pyo3::PyResult<FeatureOptions> {
    (crate::api::batch::batch_feature_options(value))
        .map(|value| FeatureOptions { inner: Some(value) })
}
#[pyfunction(name = "batch_extract_file")]
#[pyo3(signature = (stem, options, extractor, uniform))]
pub fn binding_batch_extract_file(
    stem: &str,
    options: pyo3::PyRef<'_, BatchOptions>,
    extractor: pyo3::PyRef<'_, Extractor>,
    uniform: Py<PyAny>,
) -> pyo3::PyResult<BatchOutputs> {
    (crate::api::host::batch_extract_file(
        stem,
        options.inner.as_ref().ok_or_else(consumed)?,
        extractor.inner.as_ref().ok_or_else(consumed)?,
        &crate::callbacks::Function::new(uniform)?,
    ))
    .map(|value| BatchOutputs { inner: Some(value) })
}
