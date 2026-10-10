use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Label {
    pub(crate) inner: Option<crate::api::Label>,
}
#[pymethods]
impl Label {
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
impl Label {
    #[getter(start)]
    fn binding_get_start(&self) -> PyResult<f64> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.start)
    }
    #[setter(start)]
    fn binding_set_start(&mut self, value: f64) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.start = value;
        Ok(())
    }
    #[getter(end)]
    fn binding_get_end(&self) -> PyResult<f64> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.end)
    }
    #[setter(end)]
    fn binding_set_end(&mut self, value: f64) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.end = value;
        Ok(())
    }
}
#[pymethods]
impl Label {
    #[new]
    #[pyo3(signature = (start, end, name))]
    pub fn binding_new(start: f64, end: f64, name: String) -> pyo3::PyResult<Self> {
        Ok(Label {
            inner: Some(crate::api::Label::new(start, end, name)),
        })
    }
    #[getter(name)]
    pub fn binding_name(&self) -> pyo3::PyResult<String> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).name())
    }
    #[setter(name)]
    pub fn binding_set_name(&mut self, name: String) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_name(name);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Label {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
