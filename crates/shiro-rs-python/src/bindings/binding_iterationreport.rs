use super::*;
#[pyclass(module = "shiro_rs")]
pub struct IterationReport {
    pub(crate) inner: Option<crate::api::IterationReport>,
}
#[pymethods]
impl IterationReport {
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
impl IterationReport {
    #[new]
    #[pyo3(signature = (iteration, temperature, mean_log_likelihood, rows))]
    pub fn binding_new(
        iteration: usize,
        temperature: f32,
        mean_log_likelihood: f32,
        rows: pyo3::PyRef<'_, LikelihoodRows>,
    ) -> pyo3::PyResult<Self> {
        Ok(IterationReport {
            inner: Some(crate::api::IterationReport::new(
                iteration,
                temperature,
                mean_log_likelihood,
                rows.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
    #[getter(iteration)]
    pub fn binding_iteration(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).iteration())
    }
    #[setter(iteration)]
    pub fn binding_set_iteration(&mut self, value: usize) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_iteration(value);
        Ok(())
    }
    #[getter(temperature)]
    pub fn binding_temperature(&self) -> pyo3::PyResult<f32> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).temperature())
    }
    #[setter(temperature)]
    pub fn binding_set_temperature(&mut self, value: f32) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_temperature(value);
        Ok(())
    }
    #[getter(mean_log_likelihood)]
    pub fn binding_mean_log_likelihood(&self) -> pyo3::PyResult<f32> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).mean_log_likelihood())
    }
    #[setter(mean_log_likelihood)]
    pub fn binding_set_mean_log_likelihood(&mut self, value: f32) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_mean_log_likelihood(value);
        Ok(())
    }
    #[pyo3(name = "file_likelihoods")]
    #[pyo3(signature = ())]
    pub fn binding_file_likelihoods(&self) -> pyo3::PyResult<LikelihoodRows> {
        Ok(LikelihoodRows {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).file_likelihoods()),
        })
    }
    #[pyo3(name = "set_file_likelihoods")]
    #[pyo3(signature = (rows))]
    pub fn binding_set_file_likelihoods(
        &mut self,
        rows: pyo3::PyRef<'_, LikelihoodRows>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_file_likelihoods(rows.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(IterationReport {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
