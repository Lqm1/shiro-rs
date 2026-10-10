use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Stream {
    pub(crate) inner: Option<crate::api::Stream>,
}
#[pymethods]
impl Stream {
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
impl Stream {
    #[new]
    #[pyo3(signature = (emissions, components, dimensions))]
    pub fn binding_new(
        emissions: usize,
        components: usize,
        dimensions: usize,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Stream::new(emissions, components, dimensions))
            .map(|value| Stream { inner: Some(value) })
    }
    #[getter(emissions)]
    pub fn binding_emissions(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).emissions())
    }
    #[pyo3(name = "weight")]
    #[pyo3(signature = ())]
    pub fn binding_weight(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).weight())
    }
    #[pyo3(name = "set_weight")]
    #[pyo3(signature = (values))]
    pub fn binding_set_weight(&mut self, values: Vec<f32>) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).set_weight(&values)
    }
    #[pyo3(name = "emission")]
    #[pyo3(signature = (index))]
    pub fn binding_emission(&self, index: usize) -> pyo3::PyResult<Option<Gaussian>> {
        Ok(
            ((self.inner.as_ref().ok_or_else(consumed)?).emission(index))
                .map(|value| Gaussian { inner: Some(value) }),
        )
    }
    #[pyo3(name = "set_emission")]
    #[pyo3(signature = (index, value))]
    pub fn binding_set_emission(
        &mut self,
        index: usize,
        value: pyo3::PyRef<'_, Gaussian>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .set_emission(index, value.inner.as_ref().ok_or_else(consumed)?)
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (weight, emissions))]
    pub fn binding_replace(
        &mut self,
        weight: Vec<f32>,
        emissions: pyo3::PyRef<'_, Gaussians>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .replace(&weight, emissions.inner.as_ref().ok_or_else(consumed)?)
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Stream {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
