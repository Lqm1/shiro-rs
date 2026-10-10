use super::*;
#[pyclass(module = "shiro_rs")]
pub struct PhoneMapOptions {
    pub(crate) inner: Option<crate::api::PhoneMapOptions>,
}
#[pymethods]
impl PhoneMapOptions {
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
impl PhoneMapOptions {
    #[getter(states_per_phone)]
    fn binding_get_states_per_phone(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.states_per_phone)
    }
    #[setter(states_per_phone)]
    fn binding_set_states_per_phone(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.states_per_phone = value;
        Ok(())
    }
    #[getter(streams)]
    fn binding_get_streams(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.streams)
    }
    #[setter(streams)]
    fn binding_set_streams(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.streams = value;
        Ok(())
    }
    #[getter(topology)]
    fn binding_get_topology(&self) -> PyResult<Option<String>> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.topology.clone())
    }
    #[setter(topology)]
    fn binding_set_topology(&mut self, value: Option<String>) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.topology = value;
        Ok(())
    }
    #[getter(weak_skips)]
    fn binding_get_weak_skips(&self) -> PyResult<bool> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.weak_skips)
    }
    #[setter(weak_skips)]
    fn binding_set_weak_skips(&mut self, value: bool) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.weak_skips = value;
        Ok(())
    }
}
#[pymethods]
impl PhoneMapOptions {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(PhoneMapOptions {
            inner: Some(crate::api::PhoneMapOptions::new()),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(PhoneMapOptions {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
