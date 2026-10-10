use super::*;
#[pyclass(module = "shiro_rs")]
pub struct ModelDefinition {
    pub(crate) inner: Option<crate::api::ModelDefinition>,
}
#[pymethods]
impl ModelDefinition {
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
impl ModelDefinition {
    #[new]
    #[pyo3(signature = (json))]
    pub fn binding_new(json: &str) -> pyo3::PyResult<Self> {
        (crate::api::ModelDefinition::new(json)).map(|value| ModelDefinition { inner: Some(value) })
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
        Ok(ModelDefinition {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
#[pymethods]
impl ModelDefinition {
    #[pyo3(name = "build")]
    #[pyo3(signature = ())]
    pub fn binding_build(&self) -> pyo3::PyResult<Model> {
        ((self.inner.as_ref().ok_or_else(consumed)?).build())
            .map(|value| Model { inner: Some(value) })
    }
}
