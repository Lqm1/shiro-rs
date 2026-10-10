use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Assignment {
    pub(crate) inner: Option<crate::api::Assignment>,
}
#[pymethods]
impl Assignment {
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
impl Assignment {
    #[getter(state)]
    fn binding_get_state(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.state)
    }
    #[setter(state)]
    fn binding_set_state(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.state = value;
        Ok(())
    }
    #[getter(file)]
    fn binding_get_file(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.file)
    }
    #[setter(file)]
    fn binding_set_file(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.file = value;
        Ok(())
    }
    #[getter(segment)]
    fn binding_get_segment(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.segment)
    }
    #[setter(segment)]
    fn binding_set_segment(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.segment = value;
        Ok(())
    }
}
#[pymethods]
impl Assignment {
    #[new]
    #[pyo3(signature = (state, file, segment))]
    pub fn binding_new(state: usize, file: usize, segment: usize) -> pyo3::PyResult<Self> {
        Ok(Assignment {
            inner: Some(crate::api::Assignment::new(state, file, segment)),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Assignment {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
