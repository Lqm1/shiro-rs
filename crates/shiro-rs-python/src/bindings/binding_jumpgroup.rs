use super::*;
#[pyclass(module = "shiro_rs")]
pub struct JumpGroup {
    pub(crate) inner: Option<crate::api::JumpGroup>,
}
#[pymethods]
impl JumpGroup {
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
impl JumpGroup {
    #[new]
    #[pyo3(signature = (deltas, probabilities))]
    pub fn binding_new(deltas: Vec<i32>, probabilities: Vec<f32>) -> pyo3::PyResult<Self> {
        (crate::api::JumpGroup::new(&deltas, &probabilities))
            .map(|value| JumpGroup { inner: Some(value) })
    }
    #[getter(length)]
    pub fn binding_length(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).length())
    }
    #[pyo3(name = "deltas")]
    #[pyo3(signature = ())]
    pub fn binding_deltas(&self) -> pyo3::PyResult<Vec<i32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).deltas())
    }
    #[pyo3(name = "probabilities")]
    #[pyo3(signature = ())]
    pub fn binding_probabilities(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).probabilities())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(JumpGroup {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
