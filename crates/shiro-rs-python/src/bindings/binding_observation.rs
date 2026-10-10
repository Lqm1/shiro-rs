use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Observation {
    pub(crate) inner: Option<crate::api::Observation>,
}
#[pymethods]
impl Observation {
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
impl Observation {
    #[new]
    #[pyo3(signature = (frames, dimensions))]
    pub fn binding_new(frames: usize, dimensions: Vec<u32>) -> pyo3::PyResult<Self> {
        (crate::api::Observation::new(frames, &dimensions))
            .map(|value| Observation { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "read_rawfloat")]
    #[pyo3(signature = (bytes, dimensions, maximum_frames))]
    pub fn binding_read_rawfloat(
        bytes: Vec<u8>,
        dimensions: Vec<u32>,
        maximum_frames: usize,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Observation::read_rawfloat(&bytes, &dimensions, maximum_frames))
            .map(|value| Observation { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "from_model_rawfloat")]
    #[pyo3(signature = (bytes, model, maximum_frames))]
    pub fn binding_from_model_rawfloat(
        bytes: Vec<u8>,
        model: pyo3::PyRef<'_, Model>,
        maximum_frames: usize,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Observation::from_model_rawfloat(
            &bytes,
            model.inner.as_ref().ok_or_else(consumed)?,
            maximum_frames,
        ))
        .map(|value| Observation { inner: Some(value) })
    }
    #[getter(frames)]
    pub fn binding_frames(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).frames())
    }
    #[getter(streams)]
    pub fn binding_streams(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).streams())
    }
    #[pyo3(name = "stream")]
    #[pyo3(signature = (index))]
    pub fn binding_stream(&self, index: usize) -> pyo3::PyResult<Option<ObservationStream>> {
        Ok(((self.inner.as_ref().ok_or_else(consumed)?).stream(index))
            .map(|value| ObservationStream { inner: Some(value) }))
    }
    #[pyo3(name = "set_stream")]
    #[pyo3(signature = (index, value))]
    pub fn binding_set_stream(
        &mut self,
        index: usize,
        value: pyo3::PyRef<'_, ObservationStream>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .set_stream(index, value.inner.as_ref().ok_or_else(consumed)?)
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (frames, streams))]
    pub fn binding_replace(
        &mut self,
        frames: usize,
        streams: pyo3::PyRef<'_, ObservationStreams>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .replace(frames, streams.inner.as_ref().ok_or_else(consumed)?)
    }
    #[pyo3(name = "frame")]
    #[pyo3(signature = (stream, time))]
    pub fn binding_frame(&self, stream: usize, time: usize) -> pyo3::PyResult<Option<Vec<f32>>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).frame(stream, time))
    }
    #[pyo3(name = "validate")]
    #[pyo3(signature = ())]
    pub fn binding_validate(&self) -> pyo3::PyResult<()> {
        (self.inner.as_ref().ok_or_else(consumed)?).validate()
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Observation {
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
