use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Segmentation {
    pub(crate) inner: Option<crate::api::Segmentation>,
}
#[pymethods]
impl Segmentation {
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
impl Segmentation {
    #[new]
    #[pyo3(signature = (streams, segments))]
    pub fn binding_new(streams: usize, segments: usize) -> pyo3::PyResult<Self> {
        (crate::api::Segmentation::new(streams, segments))
            .map(|value| Segmentation { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "from_states")]
    #[pyo3(signature = (states, model))]
    pub fn binding_from_states(
        states: pyo3::PyRef<'_, States>,
        model: pyo3::PyRef<'_, Model>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Segmentation::from_states(
            states.inner.as_ref().ok_or_else(consumed)?,
            model.inner.as_ref().ok_or_else(consumed)?,
        ))
        .map(|value| Segmentation { inner: Some(value) })
    }
    #[getter(streams)]
    pub fn binding_streams(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).streams())
    }
    #[getter(segments)]
    pub fn binding_segments(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).segments())
    }
    #[pyo3(name = "boundaries")]
    #[pyo3(signature = ())]
    pub fn binding_boundaries(&self) -> pyo3::PyResult<Vec<i32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).boundaries())
    }
    #[pyo3(name = "duration_states")]
    #[pyo3(signature = ())]
    pub fn binding_duration_states(&self) -> pyo3::PyResult<Vec<i32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).duration_states())
    }
    #[pyo3(name = "output_states")]
    #[pyo3(signature = (stream))]
    pub fn binding_output_states(&self, stream: usize) -> pyo3::PyResult<Vec<i32>> {
        (self.inner.as_ref().ok_or_else(consumed)?).output_states(stream)
    }
    #[pyo3(name = "set_boundaries")]
    #[pyo3(signature = (values))]
    pub fn binding_set_boundaries(&mut self, values: Vec<i32>) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).set_boundaries(&values)
    }
    #[pyo3(name = "set_duration_states")]
    #[pyo3(signature = (values))]
    pub fn binding_set_duration_states(&mut self, values: Vec<i32>) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).set_duration_states(&values)
    }
    #[pyo3(name = "set_output_states")]
    #[pyo3(signature = (stream, values))]
    pub fn binding_set_output_states(
        &mut self,
        stream: usize,
        values: Vec<i32>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).set_output_states(stream, &values)
    }
    #[pyo3(name = "outgoing")]
    #[pyo3(signature = (state))]
    pub fn binding_outgoing(&self, state: usize) -> pyo3::PyResult<Option<JumpGroup>> {
        Ok(
            ((self.inner.as_ref().ok_or_else(consumed)?).outgoing(state))
                .map(|value| JumpGroup { inner: Some(value) }),
        )
    }
    #[pyo3(name = "set_outgoing")]
    #[pyo3(signature = (state, jumps))]
    pub fn binding_set_outgoing(
        &mut self,
        state: usize,
        jumps: pyo3::PyRef<'_, JumpGroup>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .set_outgoing(state, jumps.inner.as_ref().ok_or_else(consumed)?)
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (value))]
    pub fn binding_replace(&mut self, value: pyo3::PyRef<'_, Segmentation>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .replace(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "validate")]
    #[pyo3(signature = ())]
    pub fn binding_validate(&self) -> pyo3::PyResult<()> {
        (self.inner.as_ref().ok_or_else(consumed)?).validate()
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Segmentation {
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
}
