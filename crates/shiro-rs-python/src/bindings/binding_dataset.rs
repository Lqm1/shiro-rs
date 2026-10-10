use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Dataset {
    pub(crate) inner: Option<crate::api::Dataset>,
}
#[pymethods]
impl Dataset {
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
impl Dataset {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(Dataset {
            inner: Some(crate::api::Dataset::new()),
        })
    }
    #[pyo3(name = "observations")]
    #[pyo3(signature = ())]
    pub fn binding_observations(&self) -> pyo3::PyResult<Observations> {
        Ok(Observations {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).observations()),
        })
    }
    #[pyo3(name = "segmentations")]
    #[pyo3(signature = ())]
    pub fn binding_segmentations(&self) -> pyo3::PyResult<Segmentations> {
        Ok(Segmentations {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).segmentations()),
        })
    }
    #[pyo3(name = "set_observations")]
    #[pyo3(signature = (values))]
    pub fn binding_set_observations(
        &mut self,
        values: pyo3::PyRef<'_, Observations>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_observations(values.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "set_segmentations")]
    #[pyo3(signature = (values))]
    pub fn binding_set_segmentations(
        &mut self,
        values: pyo3::PyRef<'_, Segmentations>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_segmentations(values.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (observations, segmentations))]
    pub fn binding_replace(
        &mut self,
        observations: pyo3::PyRef<'_, Observations>,
        segmentations: pyo3::PyRef<'_, Segmentations>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).replace(
            observations.inner.as_ref().ok_or_else(consumed)?,
            segmentations.inner.as_ref().ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Dataset {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
#[pymethods]
impl Dataset {
    #[staticmethod]
    #[pyo3(name = "load_files")]
    #[pyo3(signature = (document, model, maximum_frames))]
    pub fn binding_load_files(
        document: pyo3::PyRef<'_, SegmentationDocument>,
        model: pyo3::PyRef<'_, Model>,
        maximum_frames: usize,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Dataset::load_files(
            document.inner.as_ref().ok_or_else(consumed)?,
            model.inner.as_ref().ok_or_else(consumed)?,
            maximum_frames,
        ))
        .map(|value| Dataset { inner: Some(value) })
    }
}
#[pymethods]
impl Dataset {
    #[staticmethod]
    #[pyo3(name = "load")]
    #[pyo3(signature = (document, model, files, maximum_frames))]
    pub fn binding_load(
        document: pyo3::PyRef<'_, SegmentationDocument>,
        model: pyo3::PyRef<'_, Model>,
        files: pyo3::PyRef<'_, FeatureFiles>,
        maximum_frames: usize,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Dataset::load(
            document.inner.as_ref().ok_or_else(consumed)?,
            model.inner.as_ref().ok_or_else(consumed)?,
            files.inner.as_ref().ok_or_else(consumed)?,
            maximum_frames,
        ))
        .map(|value| Dataset { inner: Some(value) })
    }
}
