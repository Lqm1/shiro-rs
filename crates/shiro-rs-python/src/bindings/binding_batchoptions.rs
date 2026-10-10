use super::*;
#[pyclass(module = "shiro_rs")]
pub struct BatchOptions {
    pub(crate) inner: Option<crate::api::BatchOptions>,
}
#[pymethods]
impl BatchOptions {
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
impl BatchOptions {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(BatchOptions {
            inner: Some(crate::api::BatchOptions::new()),
        })
    }
    #[pyo3(name = "audio")]
    #[pyo3(signature = ())]
    pub fn binding_audio(&self) -> pyo3::PyResult<AudioOptions> {
        Ok(AudioOptions {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).audio()),
        })
    }
    #[pyo3(name = "set_audio")]
    #[pyo3(signature = (value))]
    pub fn binding_set_audio(
        &mut self,
        value: pyo3::PyRef<'_, AudioOptions>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_audio(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[getter(input_extension)]
    pub fn binding_input_extension(&self) -> pyo3::PyResult<String> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).input_extension())
    }
    #[setter(input_extension)]
    pub fn binding_set_input_extension(&mut self, value: String) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_input_extension(value);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(BatchOptions {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
