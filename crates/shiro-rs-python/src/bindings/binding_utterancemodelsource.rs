use super::*;
#[pyclass(module = "shiro_rs")]
pub struct UtteranceModelSource {
    pub(crate) inner: Option<crate::api::UtteranceModelSource>,
}
#[pymethods]
impl UtteranceModelSource {
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
impl UtteranceModelSource {
    #[staticmethod]
    #[pyo3(name = "fresh")]
    #[pyo3(signature = ())]
    pub fn binding_fresh() -> pyo3::PyResult<Self> {
        Ok(UtteranceModelSource {
            inner: Some(crate::api::UtteranceModelSource::fresh()),
        })
    }
    #[staticmethod]
    #[pyo3(name = "initialized")]
    #[pyo3(signature = (model))]
    pub fn binding_initialized(model: pyo3::PyRef<'_, Model>) -> pyo3::PyResult<Self> {
        Ok(UtteranceModelSource {
            inner: Some(crate::api::UtteranceModelSource::initialized(
                model.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
    #[staticmethod]
    #[pyo3(name = "trained")]
    #[pyo3(signature = (model))]
    pub fn binding_trained(model: pyo3::PyRef<'_, Model>) -> pyo3::PyResult<Self> {
        Ok(UtteranceModelSource {
            inner: Some(crate::api::UtteranceModelSource::trained(
                model.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
    #[getter(kind)]
    pub fn binding_kind(&self) -> pyo3::PyResult<u32> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).kind())
    }
    #[pyo3(name = "model")]
    #[pyo3(signature = ())]
    pub fn binding_model(&self) -> pyo3::PyResult<Option<Model>> {
        Ok(((self.inner.as_ref().ok_or_else(consumed)?).model())
            .map(|value| Model { inner: Some(value) }))
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(UtteranceModelSource {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
