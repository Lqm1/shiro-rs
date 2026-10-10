use super::*;
#[pyclass(module = "shiro_rs")]
pub struct FileLikelihoods {
    pub(crate) inner: Option<crate::api::FileLikelihoods>,
}
#[pymethods]
impl FileLikelihoods {
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
impl FileLikelihoods {
    #[new]
    #[pyo3(signature = (values))]
    pub fn binding_new(values: Vec<f32>) -> pyo3::PyResult<Self> {
        Ok(FileLikelihoods {
            inner: Some(crate::api::FileLikelihoods::new(&values)),
        })
    }
    #[pyo3(name = "values")]
    #[pyo3(signature = ())]
    pub fn binding_values(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).values())
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (values))]
    pub fn binding_replace(&mut self, values: Vec<f32>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).replace(&values);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(FileLikelihoods {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
