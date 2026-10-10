use super::*;
#[pyclass(module = "shiro_rs")]
pub struct TrainingResult {
    pub(crate) inner: Option<crate::api::TrainingResult>,
}
#[pymethods]
impl TrainingResult {
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
impl TrainingResult {
    #[new]
    #[pyo3(signature = (model, iterations))]
    pub fn binding_new(
        model: pyo3::PyRef<'_, Model>,
        iterations: pyo3::PyRef<'_, IterationReports>,
    ) -> pyo3::PyResult<Self> {
        Ok(TrainingResult {
            inner: Some(crate::api::TrainingResult::new(
                model.inner.as_ref().ok_or_else(consumed)?,
                iterations.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
    #[pyo3(name = "model")]
    #[pyo3(signature = ())]
    pub fn binding_model(&self) -> pyo3::PyResult<Model> {
        Ok(Model {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).model()),
        })
    }
    #[pyo3(name = "set_model")]
    #[pyo3(signature = (model))]
    pub fn binding_set_model(&mut self, model: pyo3::PyRef<'_, Model>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_model(model.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "iterations")]
    #[pyo3(signature = ())]
    pub fn binding_iterations(&self) -> pyo3::PyResult<IterationReports> {
        Ok(IterationReports {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).iterations()),
        })
    }
    #[pyo3(name = "write_likelihood_csv")]
    #[pyo3(signature = ())]
    pub fn binding_write_likelihood_csv(&self) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyBytes>> {
        ((self.inner.as_ref().ok_or_else(consumed)?).write_likelihood_csv()).map(|value| {
            let bytes = value;
            pyo3::Python::attach(|py| pyo3::types::PyBytes::new(py, &bytes).unbind())
        })
    }
    #[pyo3(name = "set_iterations")]
    #[pyo3(signature = (iterations))]
    pub fn binding_set_iterations(
        &mut self,
        iterations: pyo3::PyRef<'_, IterationReports>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_iterations(iterations.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(TrainingResult {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
