use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Features {
    pub(crate) inner: Option<crate::api::Features>,
}
#[pymethods]
impl Features {
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
impl Features {
    #[new]
    #[pyo3(signature = (frames, columns, values))]
    pub fn binding_new(frames: usize, columns: usize, values: Vec<f32>) -> pyo3::PyResult<Self> {
        Ok(Features {
            inner: Some(crate::api::Features::new(frames, columns, &values)),
        })
    }
    #[staticmethod]
    #[pyo3(name = "extract")]
    #[pyo3(signature = (signal, options))]
    pub fn binding_extract(
        signal: Vec<f32>,
        options: pyo3::PyRef<'_, FeatureOptions>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Features::extract(&signal, options.inner.as_ref().ok_or_else(consumed)?))
            .map(|value| Features { inner: Some(value) })
    }
    #[getter(frames)]
    pub fn binding_frames(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).frames())
    }
    #[setter(frames)]
    pub fn binding_set_frames(&mut self, value: usize) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_frames(value);
        Ok(())
    }
    #[getter(columns)]
    pub fn binding_columns(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).columns())
    }
    #[setter(columns)]
    pub fn binding_set_columns(&mut self, value: usize) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_columns(value);
        Ok(())
    }
    #[pyo3(name = "values")]
    #[pyo3(signature = ())]
    pub fn binding_values(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).values())
    }
    #[pyo3(name = "set_values")]
    #[pyo3(signature = (values))]
    pub fn binding_set_values(&mut self, values: Vec<f32>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_values(&values);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Features {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
