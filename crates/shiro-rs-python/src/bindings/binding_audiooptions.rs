use super::*;
#[pyclass(module = "shiro_rs")]
pub struct AudioOptions {
    pub(crate) inner: Option<crate::api::AudioOptions>,
}
#[pymethods]
impl AudioOptions {
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
impl AudioOptions {
    #[getter(normalize)]
    fn binding_get_normalize(&self) -> PyResult<bool> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.normalize)
    }
    #[setter(normalize)]
    fn binding_set_normalize(&mut self, value: bool) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.normalize = value;
        Ok(())
    }
    #[getter(dither_level)]
    fn binding_get_dither_level(&self) -> PyResult<f32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.dither_level)
    }
    #[setter(dither_level)]
    fn binding_set_dither_level(&mut self, value: f32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.dither_level = value;
        Ok(())
    }
    #[getter(output_sample_rate)]
    fn binding_get_output_sample_rate(&self) -> PyResult<Option<u32>> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.output_sample_rate)
    }
    #[setter(output_sample_rate)]
    fn binding_set_output_sample_rate(&mut self, value: Option<u32>) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.output_sample_rate = value;
        Ok(())
    }
    #[getter(boundary)]
    fn binding_get_boundary(&self) -> PyResult<u32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.boundary)
    }
    #[setter(boundary)]
    fn binding_set_boundary(&mut self, value: u32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.boundary = value;
        Ok(())
    }
    #[getter(kernel)]
    fn binding_get_kernel(&self) -> PyResult<u32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.kernel)
    }
    #[setter(kernel)]
    fn binding_set_kernel(&mut self, value: u32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.kernel = value;
        Ok(())
    }
}
#[pymethods]
impl AudioOptions {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(AudioOptions {
            inner: Some(crate::api::AudioOptions::new()),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(AudioOptions {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
