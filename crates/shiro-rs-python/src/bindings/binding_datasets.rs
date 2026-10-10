use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Datasets {
    pub(crate) inner: Option<crate::api::Datasets>,
}
#[pymethods]
impl Datasets {
    #[staticmethod]
    #[pyo3(name = "load_training_paths")]
    #[pyo3(signature = (document, model, maximum_frames, isolated))]
    pub fn binding_load_training_paths(
        document: pyo3::PyRef<'_, SegmentationDocument>,
        model: pyo3::PyRef<'_, Model>,
        maximum_frames: usize,
        isolated: bool,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Datasets::load_training_paths(
            document.inner.as_ref().ok_or_else(consumed)?,
            model.inner.as_ref().ok_or_else(consumed)?,
            maximum_frames,
            isolated,
        ))
        .map(|value| Datasets { inner: Some(value) })
    }
}
#[pymethods]
impl Datasets {
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
impl Datasets {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(Datasets {
            inner: Some(crate::api::Datasets::new()),
        })
    }
    #[getter(length)]
    pub fn binding_length(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).length())
    }
    #[pyo3(name = "get")]
    #[pyo3(signature = (index))]
    pub fn binding_get(&self, index: usize) -> pyo3::PyResult<Option<Dataset>> {
        Ok(((self.inner.as_ref().ok_or_else(consumed)?).get(index))
            .map(|value| Dataset { inner: Some(value) }))
    }
    #[pyo3(name = "push")]
    #[pyo3(signature = (value))]
    pub fn binding_push(&mut self, value: pyo3::PyRef<'_, Dataset>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .push(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (index, value))]
    pub fn binding_replace(
        &mut self,
        index: usize,
        value: pyo3::PyRef<'_, Dataset>,
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
        Ok(Datasets {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
#[pymethods]
impl Datasets {
    #[staticmethod]
    #[pyo3(name = "load_training_files")]
    #[pyo3(signature = (document, model, files, maximum_frames, isolated))]
    pub fn binding_load_training_files(
        document: pyo3::PyRef<'_, SegmentationDocument>,
        model: pyo3::PyRef<'_, Model>,
        files: pyo3::PyRef<'_, FeatureFiles>,
        maximum_frames: usize,
        isolated: bool,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Datasets::load_training_files(
            document.inner.as_ref().ok_or_else(consumed)?,
            model.inner.as_ref().ok_or_else(consumed)?,
            files.inner.as_ref().ok_or_else(consumed)?,
            maximum_frames,
            isolated,
        ))
        .map(|value| Datasets { inner: Some(value) })
    }
}
