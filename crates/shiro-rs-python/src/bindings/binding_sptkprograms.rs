use super::*;
#[pyclass(module = "shiro_rs")]
pub struct SptkPrograms {
    pub(crate) inner: Option<crate::api::SptkPrograms>,
}
#[pymethods]
impl SptkPrograms {
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
impl SptkPrograms {
    #[getter(frame)]
    fn binding_get_frame(&self) -> PyResult<String> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.frame.clone())
    }
    #[setter(frame)]
    fn binding_set_frame(&mut self, value: String) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.frame = value;
        Ok(())
    }
    #[getter(mfcc)]
    fn binding_get_mfcc(&self) -> PyResult<String> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.mfcc.clone())
    }
    #[setter(mfcc)]
    fn binding_set_mfcc(&mut self, value: String) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.mfcc = value;
        Ok(())
    }
    #[getter(delta)]
    fn binding_get_delta(&self) -> PyResult<String> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.delta.clone())
    }
    #[setter(delta)]
    fn binding_set_delta(&mut self, value: String) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.delta = value;
        Ok(())
    }
}
#[pymethods]
impl SptkPrograms {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(SptkPrograms {
            inner: Some(crate::api::SptkPrograms::new()),
        })
    }
}
