use super::*;
#[pyclass(module = "shiro_rs")]
pub struct PhoneMap {
    pub(crate) inner: Option<crate::api::PhoneMap>,
}
#[pymethods]
impl PhoneMap {
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
impl PhoneMap {
    #[new]
    #[pyo3(signature = (json))]
    pub fn binding_new(json: &str) -> pyo3::PyResult<Self> {
        (crate::api::PhoneMap::new(json)).map(|value| PhoneMap { inner: Some(value) })
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
        Ok(PhoneMap {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
#[pymethods]
impl PhoneMap {
    #[staticmethod]
    #[pyo3(name = "create")]
    #[pyo3(signature = (text, options))]
    pub fn binding_create(
        text: &str,
        options: pyo3::PyRef<'_, PhoneMapOptions>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::PhoneMap::create(text, options.inner.as_ref().ok_or_else(consumed)?))
            .map(|value| PhoneMap { inner: Some(value) })
    }
    #[pyo3(name = "to_definition")]
    #[pyo3(signature = (dimensions, hop))]
    pub fn binding_to_definition(
        &self,
        dimensions: usize,
        hop: f64,
    ) -> pyo3::PyResult<ModelDefinition> {
        ((self.inner.as_ref().ok_or_else(consumed)?).to_definition(dimensions, hop))
            .map(|value| ModelDefinition { inner: Some(value) })
    }
    #[pyo3(name = "initial")]
    #[pyo3(signature = (phones, frames))]
    pub fn binding_initial(&self, phones: Vec<String>, frames: usize) -> pyo3::PyResult<States> {
        ((self.inner.as_ref().ok_or_else(consumed)?).initial(phones, frames))
            .map(|value| States { inner: Some(value) })
    }
}
