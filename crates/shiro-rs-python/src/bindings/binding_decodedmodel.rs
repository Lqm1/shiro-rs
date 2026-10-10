use super::*;
#[pyclass(module = "shiro_rs")]
pub struct DecodedModel {
    pub(crate) inner: Option<crate::api::DecodedModel>,
}
#[pymethods]
impl DecodedModel {
    fn close(&mut self) {
        self.inner = None;
    }
    fn free(&mut self) {
        self.inner = None;
    }
    #[getter]
    fn is_closed(&self) -> bool {
        self.inner.is_none()
    }
}
#[pymethods]
impl DecodedModel {
    #[getter(position)]
    pub fn binding_position(&self) -> pyo3::PyResult<u64> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).position())
    }
    #[pyo3(name = "parameters")]
    #[pyo3(signature = ())]
    pub fn binding_parameters(&self) -> pyo3::PyResult<Model> {
        Ok(Model {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).parameters()),
        })
    }
    #[pyo3(name = "into_parameters")]
    #[pyo3(signature = ())]
    pub fn binding_into_parameters(&mut self) -> pyo3::PyResult<Model> {
        Ok(Model {
            inner: Some((self.inner.take().ok_or_else(consumed)?).into_parameters()),
        })
    }
}
