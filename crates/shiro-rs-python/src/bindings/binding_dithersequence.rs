use super::*;
#[pyclass(module = "shiro_rs")]
pub struct DitherSequence {
    pub(crate) inner: Option<crate::api::DitherSequence>,
}
#[pymethods]
impl DitherSequence {
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
impl DitherSequence {
    #[staticmethod]
    #[pyo3(name = "windows")]
    #[pyo3(signature = ())]
    pub fn binding_windows() -> pyo3::PyResult<Self> {
        Ok(DitherSequence {
            inner: Some(crate::api::DitherSequence::windows()),
        })
    }
    #[staticmethod]
    #[pyo3(name = "linux_gnu")]
    #[pyo3(signature = ())]
    pub fn binding_linux_gnu() -> pyo3::PyResult<Self> {
        Ok(DitherSequence {
            inner: Some(crate::api::DitherSequence::linux_gnu()),
        })
    }
    #[pyo3(name = "next_uniform")]
    #[pyo3(signature = ())]
    pub fn binding_next_uniform(&mut self) -> pyo3::PyResult<f32> {
        Ok((self.inner.as_mut().ok_or_else(consumed)?).next_uniform())
    }
}
