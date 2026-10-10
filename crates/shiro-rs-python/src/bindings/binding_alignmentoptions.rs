use super::*;
#[pyclass(module = "shiro_rs")]
pub struct AlignmentOptions {
    pub(crate) inner: Option<crate::api::AlignmentOptions>,
}
#[pymethods]
impl AlignmentOptions {
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
impl AlignmentOptions {
    #[getter(duration_mode)]
    fn binding_get_duration_mode(&self) -> PyResult<u32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.duration_mode)
    }
    #[setter(duration_mode)]
    fn binding_set_duration_mode(&mut self, value: u32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.duration_mode = value;
        Ok(())
    }
    #[getter(isolated)]
    fn binding_get_isolated(&self) -> PyResult<bool> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.isolated)
    }
    #[setter(isolated)]
    fn binding_set_isolated(&mut self, value: bool) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.isolated = value;
        Ok(())
    }
    #[getter(hsmm_temperature)]
    fn binding_get_hsmm_temperature(&self) -> PyResult<f32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.hsmm_temperature)
    }
    #[setter(hsmm_temperature)]
    fn binding_set_hsmm_temperature(&mut self, value: f32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.hsmm_temperature = value;
        Ok(())
    }
    #[getter(duration_weight)]
    fn binding_get_duration_weight(&self) -> PyResult<f32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.duration_weight)
    }
    #[setter(duration_weight)]
    fn binding_set_duration_weight(&mut self, value: f32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.duration_weight = value;
        Ok(())
    }
    #[getter(state_radius)]
    fn binding_get_state_radius(&self) -> PyResult<f32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.state_radius)
    }
    #[setter(state_radius)]
    fn binding_set_state_radius(&mut self, value: f32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.state_radius = value;
        Ok(())
    }
    #[getter(duration_extra)]
    fn binding_get_duration_extra(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.duration_extra)
    }
    #[setter(duration_extra)]
    fn binding_set_duration_extra(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.duration_extra = value;
        Ok(())
    }
    #[getter(duration_extra_factor)]
    fn binding_get_duration_extra_factor(&self) -> PyResult<f32> {
        Ok(self
            .inner
            .as_ref()
            .ok_or_else(consumed)?
            .duration_extra_factor)
    }
    #[setter(duration_extra_factor)]
    fn binding_set_duration_extra_factor(&mut self, value: f32) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(consumed)?
            .duration_extra_factor = value;
        Ok(())
    }
    #[getter(geometric_temperature)]
    fn binding_get_geometric_temperature(&self) -> PyResult<f32> {
        Ok(self
            .inner
            .as_ref()
            .ok_or_else(consumed)?
            .geometric_temperature)
    }
    #[setter(geometric_temperature)]
    fn binding_set_geometric_temperature(&mut self, value: f32) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(consumed)?
            .geometric_temperature = value;
        Ok(())
    }
    #[getter(pruning_slope)]
    fn binding_get_pruning_slope(&self) -> PyResult<f32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.pruning_slope)
    }
    #[setter(pruning_slope)]
    fn binding_set_pruning_slope(&mut self, value: f32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.pruning_slope = value;
        Ok(())
    }
}
#[pymethods]
impl AlignmentOptions {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(AlignmentOptions {
            inner: Some(crate::api::AlignmentOptions::new()),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(AlignmentOptions {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
