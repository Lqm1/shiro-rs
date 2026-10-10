use super::*;
#[pyclass(module = "shiro_rs")]
pub struct FeatureFiles {
    pub(crate) inner: Option<crate::api::FeatureFiles>,
}
#[pymethods]
impl FeatureFiles {
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
impl FeatureFiles {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(FeatureFiles {
            inner: Some(crate::api::FeatureFiles::new()),
        })
    }
    #[getter(length)]
    pub fn binding_length(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).length())
    }
    #[pyo3(name = "names")]
    #[pyo3(signature = ())]
    pub fn binding_names(&self) -> pyo3::PyResult<Vec<String>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).names())
    }
    #[pyo3(name = "get")]
    #[pyo3(signature = (filename))]
    pub fn binding_get(
        &self,
        filename: &str,
    ) -> pyo3::PyResult<Option<pyo3::Py<pyo3::types::PyBytes>>> {
        Ok(
            ((self.inner.as_ref().ok_or_else(consumed)?).get(filename)).map(|value| {
                let bytes = value;
                pyo3::Python::attach(|py| pyo3::types::PyBytes::new(py, &bytes).unbind())
            }),
        )
    }
    #[pyo3(name = "set")]
    #[pyo3(signature = (filename, bytes))]
    pub fn binding_set(&mut self, filename: &str, bytes: Vec<u8>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set(filename, &bytes);
        Ok(())
    }
    #[pyo3(name = "remove")]
    #[pyo3(signature = (filename))]
    pub fn binding_remove(&mut self, filename: &str) -> pyo3::PyResult<bool> {
        Ok((self.inner.as_mut().ok_or_else(consumed)?).remove(filename))
    }
    #[pyo3(name = "clear")]
    #[pyo3(signature = ())]
    pub fn binding_clear(&mut self) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).clear();
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(FeatureFiles {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
