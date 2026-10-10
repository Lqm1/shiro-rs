use super::*;
#[pyclass(module = "shiro_rs")]
pub struct ObservationStream {
    pub(crate) inner: Option<crate::api::ObservationStream>,
}
#[pymethods]
impl ObservationStream {
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
impl ObservationStream {
    #[new]
    #[pyo3(signature = (dimensions, values))]
    pub fn binding_new(dimensions: usize, values: Vec<f32>) -> pyo3::PyResult<Self> {
        Ok(ObservationStream {
            inner: Some(crate::api::ObservationStream::new(dimensions, &values)),
        })
    }
    #[getter(dimensions)]
    pub fn binding_dimensions(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).dimensions())
    }
    #[pyo3(name = "values")]
    #[pyo3(signature = ())]
    pub fn binding_values(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).values())
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (dimensions, values))]
    pub fn binding_replace(&mut self, dimensions: usize, values: Vec<f32>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).replace(dimensions, &values);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(ObservationStream {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
