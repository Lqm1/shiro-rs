use super::*;
#[pyclass(module = "shiro_rs")]
pub struct FeatureOptions {
    pub(crate) inner: Option<crate::api::FeatureOptions>,
}
#[pymethods]
impl FeatureOptions {
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
impl FeatureOptions {
    #[getter(kind)]
    fn binding_get_kind(&self) -> PyResult<u32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.kind)
    }
    #[setter(kind)]
    fn binding_set_kind(&mut self, value: u32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.kind = value;
        Ok(())
    }
    #[getter(order)]
    fn binding_get_order(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.order)
    }
    #[setter(order)]
    fn binding_set_order(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.order = value;
        Ok(())
    }
    #[getter(channels)]
    fn binding_get_channels(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.channels)
    }
    #[setter(channels)]
    fn binding_set_channels(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.channels = value;
        Ok(())
    }
    #[getter(frame_length)]
    fn binding_get_frame_length(&self) -> PyResult<usize> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.frame_length)
    }
    #[setter(frame_length)]
    fn binding_set_frame_length(&mut self, value: usize) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.frame_length = value;
        Ok(())
    }
    #[getter(hop)]
    fn binding_get_hop(&self) -> PyResult<f32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.hop)
    }
    #[setter(hop)]
    fn binding_set_hop(&mut self, value: f32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.hop = value;
        Ok(())
    }
    #[getter(sample_rate_hz)]
    fn binding_get_sample_rate_hz(&self) -> PyResult<f32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.sample_rate_hz)
    }
    #[setter(sample_rate_hz)]
    fn binding_set_sample_rate_hz(&mut self, value: f32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.sample_rate_hz = value;
        Ok(())
    }
    #[getter(minimum_bandwidth_hz)]
    fn binding_get_minimum_bandwidth_hz(&self) -> PyResult<f32> {
        Ok(self
            .inner
            .as_ref()
            .ok_or_else(consumed)?
            .minimum_bandwidth_hz)
    }
    #[setter(minimum_bandwidth_hz)]
    fn binding_set_minimum_bandwidth_hz(&mut self, value: f32) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(consumed)?
            .minimum_bandwidth_hz = value;
        Ok(())
    }
    #[getter(warp)]
    fn binding_get_warp(&self) -> PyResult<f32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.warp)
    }
    #[setter(warp)]
    fn binding_set_warp(&mut self, value: f32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.warp = value;
        Ok(())
    }
    #[getter(include_dc)]
    fn binding_get_include_dc(&self) -> PyResult<bool> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.include_dc)
    }
    #[setter(include_dc)]
    fn binding_set_include_dc(&mut self, value: bool) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.include_dc = value;
        Ok(())
    }
    #[getter(energy)]
    fn binding_get_energy(&self) -> PyResult<u32> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.energy)
    }
    #[setter(energy)]
    fn binding_set_energy(&mut self, value: u32) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.energy = value;
        Ok(())
    }
    #[getter(delta)]
    fn binding_get_delta(&self) -> PyResult<bool> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.delta)
    }
    #[setter(delta)]
    fn binding_set_delta(&mut self, value: bool) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.delta = value;
        Ok(())
    }
    #[getter(acceleration)]
    fn binding_get_acceleration(&self) -> PyResult<bool> {
        Ok(self.inner.as_ref().ok_or_else(consumed)?.acceleration)
    }
    #[setter(acceleration)]
    fn binding_set_acceleration(&mut self, value: bool) -> PyResult<()> {
        self.inner.as_mut().ok_or_else(consumed)?.acceleration = value;
        Ok(())
    }
}
#[pymethods]
impl FeatureOptions {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(FeatureOptions {
            inner: Some(crate::api::FeatureOptions::new()),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(FeatureOptions {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
