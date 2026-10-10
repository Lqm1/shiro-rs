use super::*;
#[pyclass(module = "shiro_rs")]
pub struct LikelihoodRows {
    pub(crate) inner: Option<crate::api::LikelihoodRows>,
}
#[pymethods]
impl LikelihoodRows {
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
impl LikelihoodRows {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(LikelihoodRows {
            inner: Some(crate::api::LikelihoodRows::new()),
        })
    }
    #[getter(length)]
    pub fn binding_length(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).length())
    }
    #[pyo3(name = "get")]
    #[pyo3(signature = (index))]
    pub fn binding_get(&self, index: usize) -> pyo3::PyResult<Option<FileLikelihoods>> {
        Ok(((self.inner.as_ref().ok_or_else(consumed)?).get(index))
            .map(|value| FileLikelihoods { inner: Some(value) }))
    }
    #[pyo3(name = "push")]
    #[pyo3(signature = (value))]
    pub fn binding_push(&mut self, value: pyo3::PyRef<'_, FileLikelihoods>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .push(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (index, value))]
    pub fn binding_replace(
        &mut self,
        index: usize,
        value: pyo3::PyRef<'_, FileLikelihoods>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .replace(index, value.inner.as_ref().ok_or_else(consumed)?)
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
        Ok(LikelihoodRows {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
