use super::*;
#[pyclass(module = "shiro_rs")]
pub struct UntiedModel {
    pub(crate) inner: Option<crate::api::UntiedModel>,
}
#[pymethods]
impl UntiedModel {
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
impl UntiedModel {
    #[new]
    #[pyo3(signature = (model, segmentation, assignments))]
    pub fn binding_new(
        model: pyo3::PyRef<'_, Model>,
        segmentation: pyo3::PyRef<'_, SegmentationDocument>,
        assignments: pyo3::PyRef<'_, Assignments>,
    ) -> pyo3::PyResult<Self> {
        Ok(UntiedModel {
            inner: Some(crate::api::UntiedModel::new(
                model.inner.as_ref().ok_or_else(consumed)?,
                segmentation.inner.as_ref().ok_or_else(consumed)?,
                assignments.inner.as_ref().ok_or_else(consumed)?,
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
    #[pyo3(signature = (value))]
    pub fn binding_set_model(&mut self, value: pyo3::PyRef<'_, Model>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_model(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "segmentation")]
    #[pyo3(signature = ())]
    pub fn binding_segmentation(&self) -> pyo3::PyResult<SegmentationDocument> {
        Ok(SegmentationDocument {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).segmentation()),
        })
    }
    #[pyo3(name = "set_segmentation")]
    #[pyo3(signature = (value))]
    pub fn binding_set_segmentation(
        &mut self,
        value: pyo3::PyRef<'_, SegmentationDocument>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_segmentation(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "assignments")]
    #[pyo3(signature = ())]
    pub fn binding_assignments(&self) -> pyo3::PyResult<Assignments> {
        Ok(Assignments {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).assignments()),
        })
    }
    #[pyo3(name = "set_assignments")]
    #[pyo3(signature = (value))]
    pub fn binding_set_assignments(
        &mut self,
        value: pyo3::PyRef<'_, Assignments>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_assignments(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(UntiedModel {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
    #[pyo3(name = "write_summary")]
    #[pyo3(signature = ())]
    pub fn binding_write_summary(&self) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyBytes>> {
        ((self.inner.as_ref().ok_or_else(consumed)?).write_summary()).map(|value| {
            let bytes = value;
            pyo3::Python::attach(|py| pyo3::types::PyBytes::new(py, &bytes).unbind())
        })
    }
}
