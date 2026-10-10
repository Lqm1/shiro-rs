use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Gaussian {
    pub(crate) inner: Option<crate::api::Gaussian>,
}
#[pymethods]
impl Gaussian {
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
impl Gaussian {
    #[new]
    #[pyo3(signature = (components, dimensions))]
    pub fn binding_new(components: usize, dimensions: usize) -> pyo3::PyResult<Self> {
        (crate::api::Gaussian::new(components, dimensions))
            .map(|value| Gaussian { inner: Some(value) })
    }
    #[getter(dimensions)]
    pub fn binding_dimensions(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).dimensions())
    }
    #[getter(components)]
    pub fn binding_components(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).components())
    }
    #[pyo3(name = "weights")]
    #[pyo3(signature = ())]
    pub fn binding_weights(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).weights())
    }
    #[pyo3(name = "means")]
    #[pyo3(signature = ())]
    pub fn binding_means(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).means())
    }
    #[pyo3(name = "variances")]
    #[pyo3(signature = ())]
    pub fn binding_variances(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).variances())
    }
    #[pyo3(name = "variance_floors")]
    #[pyo3(signature = ())]
    pub fn binding_variance_floors(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).variance_floors())
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (dimensions, weights, means, variances, variance_floors))]
    pub fn binding_replace(
        &mut self,
        dimensions: usize,
        weights: Vec<f32>,
        means: Vec<f32>,
        variances: Vec<f32>,
        variance_floors: Vec<f32>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).replace(
            dimensions,
            &weights,
            &means,
            &variances,
            &variance_floors,
        )
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Gaussian {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
