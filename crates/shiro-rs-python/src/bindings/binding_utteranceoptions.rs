use super::*;
#[pyclass(module = "shiro_rs")]
pub struct UtteranceOptions {
    pub(crate) inner: Option<crate::api::UtteranceOptions>,
}
#[pymethods]
impl UtteranceOptions {
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
impl UtteranceOptions {
    #[getter(utterances)]
    fn binding_get_utterances(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.utterances)
    }
    #[setter(utterances)]
    fn binding_set_utterances(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.utterances = value;
        Ok(())
    }
    #[getter(hop_seconds)]
    fn binding_get_hop_seconds(&self) -> PyResult<f64> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.hop_seconds)
    }
    #[setter(hop_seconds)]
    fn binding_set_hop_seconds(&mut self, value: f64) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.hop_seconds = value;
        Ok(())
    }
    #[getter(minimum_silence_seconds)]
    fn binding_get_minimum_silence_seconds(&self) -> PyResult<f64> {
        Ok(self
            .inner
            .as_ref()
            .ok_or_else(consumed)?
            .minimum_silence_seconds)
    }
    #[setter(minimum_silence_seconds)]
    fn binding_set_minimum_silence_seconds(&mut self, value: f64) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(consumed)?
            .minimum_silence_seconds = value;
        Ok(())
    }
    #[getter(minimum_voicing_seconds)]
    fn binding_get_minimum_voicing_seconds(&self) -> PyResult<f64> {
        Ok(self
            .inner
            .as_ref()
            .ok_or_else(consumed)?
            .minimum_voicing_seconds)
    }
    #[setter(minimum_voicing_seconds)]
    fn binding_set_minimum_voicing_seconds(&mut self, value: f64) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(consumed)?
            .minimum_voicing_seconds = value;
        Ok(())
    }
    #[getter(iterations)]
    fn binding_get_iterations(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.iterations)
    }
    #[setter(iterations)]
    fn binding_set_iterations(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.iterations = value;
        Ok(())
    }
}
#[pymethods]
impl UtteranceOptions {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(UtteranceOptions {
            inner: Some(crate::api::UtteranceOptions::new()),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(UtteranceOptions {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
