use super::*;
#[pyclass(module = "shiro_rs")]
pub struct IsolatedGroup {
    pub(crate) inner: Option<crate::api::IsolatedGroup>,
}
#[pymethods]
impl IsolatedGroup {
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
impl IsolatedGroup {
    #[new]
    #[pyo3(signature = (first_state, first_frame, observation, states))]
    pub fn binding_new(
        first_state: usize,
        first_frame: usize,
        observation: pyo3::PyRef<'_, Observation>,
        states: pyo3::PyRef<'_, States>,
    ) -> pyo3::PyResult<Self> {
        Ok(IsolatedGroup {
            inner: Some(crate::api::IsolatedGroup::new(
                first_state,
                first_frame,
                observation.inner.as_ref().ok_or_else(consumed)?,
                states.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
    #[getter(first_state)]
    pub fn binding_first_state(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).first_state())
    }
    #[setter(first_state)]
    pub fn binding_set_first_state(&mut self, value: usize) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_first_state(value);
        Ok(())
    }
    #[getter(first_frame)]
    pub fn binding_first_frame(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).first_frame())
    }
    #[setter(first_frame)]
    pub fn binding_set_first_frame(&mut self, value: usize) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_first_frame(value);
        Ok(())
    }
    #[pyo3(name = "observation")]
    #[pyo3(signature = ())]
    pub fn binding_observation(&self) -> pyo3::PyResult<Observation> {
        Ok(Observation {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).observation()),
        })
    }
    #[pyo3(name = "set_observation")]
    #[pyo3(signature = (value))]
    pub fn binding_set_observation(
        &mut self,
        value: pyo3::PyRef<'_, Observation>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_observation(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "states")]
    #[pyo3(signature = ())]
    pub fn binding_states(&self) -> pyo3::PyResult<States> {
        Ok(States {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).states()),
        })
    }
    #[pyo3(name = "set_states")]
    #[pyo3(signature = (value))]
    pub fn binding_set_states(&mut self, value: pyo3::PyRef<'_, States>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_states(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(IsolatedGroup {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
