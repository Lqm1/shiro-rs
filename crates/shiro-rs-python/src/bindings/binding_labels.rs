use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Labels {
    pub(crate) inner: Option<crate::api::Labels>,
}
#[pymethods]
impl Labels {
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
impl Labels {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(Labels {
            inner: Some(crate::api::Labels::new()),
        })
    }
    #[staticmethod]
    #[pyo3(name = "parse")]
    #[pyo3(signature = (text))]
    pub fn binding_parse(text: &str) -> pyo3::PyResult<Self> {
        (crate::api::Labels::parse(text)).map(|value| Labels { inner: Some(value) })
    }
    #[getter(length)]
    pub fn binding_length(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).length())
    }
    #[pyo3(name = "get")]
    #[pyo3(signature = (index))]
    pub fn binding_get(&self, index: usize) -> pyo3::PyResult<Option<Label>> {
        Ok(((self.inner.as_ref().ok_or_else(consumed)?).get(index))
            .map(|value| Label { inner: Some(value) }))
    }
    #[pyo3(name = "push")]
    #[pyo3(signature = (label))]
    pub fn binding_push(&mut self, label: pyo3::PyRef<'_, Label>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .push(label.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (index, label))]
    pub fn binding_replace(
        &mut self,
        index: usize,
        label: pyo3::PyRef<'_, Label>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .replace(index, label.inner.as_ref().ok_or_else(consumed)?)
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
        Ok(Labels {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
    #[pyo3(name = "write")]
    #[pyo3(signature = ())]
    pub fn binding_write(&self) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyBytes>> {
        ((self.inner.as_ref().ok_or_else(consumed)?).write()).map(|value| {
            let bytes = value;
            pyo3::Python::attach(|py| pyo3::types::PyBytes::new(py, &bytes).unbind())
        })
    }
    #[pyo3(name = "to_states")]
    #[pyo3(signature = (map, hop))]
    pub fn binding_to_states(
        &self,
        map: pyo3::PyRef<'_, PhoneMap>,
        hop: f64,
    ) -> pyo3::PyResult<States> {
        ((self.inner.as_ref().ok_or_else(consumed)?)
            .to_states(map.inner.as_ref().ok_or_else(consumed)?, hop))
        .map(|value| States { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "from_states")]
    #[pyo3(signature = (states, hop, include_states))]
    pub fn binding_from_states(
        states: pyo3::PyRef<'_, States>,
        hop: f64,
        include_states: bool,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Labels::from_states(
            states.inner.as_ref().ok_or_else(consumed)?,
            hop,
            include_states,
        ))
        .map(|value| Labels { inner: Some(value) })
    }
}
