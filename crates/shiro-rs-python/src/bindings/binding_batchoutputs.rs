use super::*;
#[pyclass(module = "shiro_rs")]
pub struct BatchOutputs {
    pub(crate) inner: Option<crate::api::BatchOutputs>,
}
#[pymethods]
impl BatchOutputs {
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
impl BatchOutputs {
    #[getter(raw)]
    fn binding_get_raw(&self) -> PyResult<String> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.raw.clone())
    }
    #[setter(raw)]
    fn binding_set_raw(&mut self, value: String) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.raw = value;
        Ok(())
    }
    #[getter(parameters)]
    fn binding_get_parameters(&self) -> PyResult<String> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.parameters.clone())
    }
    #[setter(parameters)]
    fn binding_set_parameters(&mut self, value: String) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.parameters = value;
        Ok(())
    }
    #[getter(mfcc)]
    fn binding_get_mfcc(&self) -> PyResult<Option<String>> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.mfcc.clone())
    }
    #[setter(mfcc)]
    fn binding_set_mfcc(&mut self, value: Option<String>) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.mfcc = value;
        Ok(())
    }
}
#[pymethods]
impl BatchOutputs {
    #[new]
    #[pyo3(signature = (raw, parameters, mfcc))]
    pub fn binding_new(
        raw: String,
        parameters: String,
        mfcc: Option<String>,
    ) -> pyo3::PyResult<Self> {
        Ok(BatchOutputs {
            inner: Some(crate::api::BatchOutputs::new(raw, parameters, mfcc)),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(BatchOutputs {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
