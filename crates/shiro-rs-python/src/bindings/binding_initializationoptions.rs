use super::*;
#[pyclass(module = "shiro_rs")]
pub struct InitializationOptions {
    pub(crate) inner: Option<crate::api::InitializationOptions>,
}
#[pymethods]
impl InitializationOptions {
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
impl InitializationOptions {
    #[getter(flat_start)]
    fn binding_get_flat_start(&self) -> PyResult<bool> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.flat_start)
    }
    #[setter(flat_start)]
    fn binding_set_flat_start(&mut self, value: bool) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.flat_start = value;
        Ok(())
    }
    #[getter(globally_tied)]
    fn binding_get_globally_tied(&self) -> PyResult<bool> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.globally_tied)
    }
    #[setter(globally_tied)]
    fn binding_set_globally_tied(&mut self, value: bool) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.globally_tied = value;
        Ok(())
    }
    #[getter(variance_floor_ratio)]
    fn binding_get_variance_floor_ratio(&self) -> PyResult<f32> {
        Ok(self
            .inner
            .as_ref()
            .ok_or_else(consumed)?
            .variance_floor_ratio)
    }
    #[setter(variance_floor_ratio)]
    fn binding_set_variance_floor_ratio(&mut self, value: f32) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(consumed)?
            .variance_floor_ratio = value;
        Ok(())
    }
}
#[pymethods]
impl InitializationOptions {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(InitializationOptions {
            inner: Some(crate::api::InitializationOptions::new()),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(InitializationOptions {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
