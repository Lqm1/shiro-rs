use super::*;
#[pyclass(module = "shiro_rs")]
pub struct States {
    pub(crate) inner: Option<crate::api::States>,
}
#[pymethods]
impl States {
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
impl States {
    #[new]
    #[pyo3(signature = (json))]
    pub fn binding_new(json: &str) -> pyo3::PyResult<Self> {
        (crate::api::States::new(json)).map(|value| States { inner: Some(value) })
    }
    #[pyo3(name = "json")]
    #[pyo3(signature = ())]
    pub fn binding_json(&self) -> pyo3::PyResult<String> {
        (self.inner.as_ref().ok_or_else(consumed)?).json()
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (json))]
    pub fn binding_replace(&mut self, json: &str) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).replace(json)
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(States {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
