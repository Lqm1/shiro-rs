use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Model {
    pub(crate) inner: Option<crate::api::Model>,
}
#[pymethods]
impl Model {
    #[pyo3(name = "align_states")]
    #[pyo3(signature = (observation, states, options))]
    pub fn binding_align_states(
        &self,
        observation: pyo3::PyRef<'_, Observation>,
        states: pyo3::PyRef<'_, States>,
        options: pyo3::PyRef<'_, AlignmentOptions>,
    ) -> pyo3::PyResult<States> {
        ((self.inner.as_ref().ok_or_else(consumed)?).align_states(
            observation.inner.as_ref().ok_or_else(consumed)?,
            states.inner.as_ref().ok_or_else(consumed)?,
            options.inner.as_ref().ok_or_else(consumed)?,
        ))
        .map(|value| States { inner: Some(value) })
    }
    #[pyo3(name = "align_document")]
    #[pyo3(signature = (document, files, maximum_frames, options))]
    pub fn binding_align_document(
        &self,
        document: pyo3::PyRef<'_, SegmentationDocument>,
        files: pyo3::PyRef<'_, FeatureFiles>,
        maximum_frames: usize,
        options: pyo3::PyRef<'_, AlignmentOptions>,
    ) -> pyo3::PyResult<SegmentationDocument> {
        ((self.inner.as_ref().ok_or_else(consumed)?).align_document(
            document.inner.as_ref().ok_or_else(consumed)?,
            files.inner.as_ref().ok_or_else(consumed)?,
            maximum_frames,
            options.inner.as_ref().ok_or_else(consumed)?,
        ))
        .map(|value| SegmentationDocument { inner: Some(value) })
    }
}
#[pymethods]
impl Model {
    #[staticmethod]
    #[pyo3(name = "read_file")]
    #[pyo3(signature = (path))]
    pub fn binding_read_file(path: &str) -> pyo3::PyResult<Self> {
        (crate::api::Model::read_file(path)).map(|value| Model { inner: Some(value) })
    }
    #[pyo3(name = "write_file")]
    #[pyo3(signature = (path, encoding))]
    pub fn binding_write_file(&self, path: &str, encoding: u32) -> pyo3::PyResult<()> {
        (self.inner.as_ref().ok_or_else(consumed)?).write_file(path, encoding)
    }
    #[pyo3(name = "align_document_files")]
    #[pyo3(signature = (document, options))]
    pub fn binding_align_document_files(
        &self,
        document: pyo3::PyRef<'_, SegmentationDocument>,
        options: pyo3::PyRef<'_, AlignmentOptions>,
    ) -> pyo3::PyResult<SegmentationDocument> {
        ((self.inner.as_ref().ok_or_else(consumed)?).align_document_files(
            document.inner.as_ref().ok_or_else(consumed)?,
            options.inner.as_ref().ok_or_else(consumed)?,
        ))
        .map(|value| SegmentationDocument { inner: Some(value) })
    }
}
#[pymethods]
impl Model {
    #[pyo3(name = "initialize")]
    #[pyo3(signature = (dataset, options))]
    pub fn binding_initialize(
        &self,
        dataset: pyo3::PyRef<'_, Dataset>,
        options: pyo3::PyRef<'_, InitializationOptions>,
    ) -> pyo3::PyResult<Model> {
        ((self.inner.as_ref().ok_or_else(consumed)?).initialize(
            dataset.inner.as_ref().ok_or_else(consumed)?,
            options.inner.as_ref().ok_or_else(consumed)?,
        ))
        .map(|value| Model { inner: Some(value) })
    }
}
#[pymethods]
impl Model {
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
impl Model {
    #[new]
    #[pyo3(signature = ())]
    pub fn binding_new() -> pyo3::PyResult<Self> {
        Ok(Model {
            inner: Some(crate::api::Model::new()),
        })
    }
    #[getter(streams)]
    pub fn binding_streams(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).streams())
    }
    #[getter(durations)]
    pub fn binding_durations(&self) -> pyo3::PyResult<usize> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).durations())
    }
    #[pyo3(name = "stream")]
    #[pyo3(signature = (index))]
    pub fn binding_stream(&self, index: usize) -> pyo3::PyResult<Option<Stream>> {
        Ok(((self.inner.as_ref().ok_or_else(consumed)?).stream(index))
            .map(|value| Stream { inner: Some(value) }))
    }
    #[pyo3(name = "duration")]
    #[pyo3(signature = (index))]
    pub fn binding_duration(&self, index: usize) -> pyo3::PyResult<Option<Duration>> {
        Ok(
            ((self.inner.as_ref().ok_or_else(consumed)?).duration(index))
                .map(|value| Duration { inner: Some(value) }),
        )
    }
    #[pyo3(name = "set_stream")]
    #[pyo3(signature = (index, value))]
    pub fn binding_set_stream(
        &mut self,
        index: usize,
        value: pyo3::PyRef<'_, Stream>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .set_stream(index, value.inner.as_ref().ok_or_else(consumed)?)
    }
    #[pyo3(name = "set_duration")]
    #[pyo3(signature = (index, value))]
    pub fn binding_set_duration(
        &mut self,
        index: usize,
        value: pyo3::PyRef<'_, Duration>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?)
            .set_duration(index, value.inner.as_ref().ok_or_else(consumed)?)
    }
    #[pyo3(name = "replace")]
    #[pyo3(signature = (streams, durations))]
    pub fn binding_replace(
        &mut self,
        streams: pyo3::PyRef<'_, Streams>,
        durations: pyo3::PyRef<'_, Durations>,
    ) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).replace(
            streams.inner.as_ref().ok_or_else(consumed)?,
            durations.inner.as_ref().ok_or_else(consumed)?,
        )
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Model {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
    #[pyo3(name = "validate")]
    #[pyo3(signature = ())]
    pub fn binding_validate(&self) -> pyo3::PyResult<()> {
        (self.inner.as_ref().ok_or_else(consumed)?).validate()
    }
    #[pyo3(name = "dimensions")]
    #[pyo3(signature = ())]
    pub fn binding_dimensions(&self) -> pyo3::PyResult<Vec<u32>> {
        (self.inner.as_ref().ok_or_else(consumed)?).dimensions()
    }
    #[staticmethod]
    #[pyo3(name = "read")]
    #[pyo3(signature = (bytes))]
    pub fn binding_read(bytes: Vec<u8>) -> pyo3::PyResult<Self> {
        (crate::api::Model::read(&bytes)).map(|value| Model { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "read_with_limits")]
    #[pyo3(signature = (bytes, max_array_entries))]
    pub fn binding_read_with_limits(
        bytes: Vec<u8>,
        max_array_entries: usize,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Model::read_with_limits(&bytes, max_array_entries))
            .map(|value| Model { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "read_prefix")]
    #[pyo3(signature = (bytes))]
    pub fn binding_read_prefix(bytes: Vec<u8>) -> pyo3::PyResult<DecodedModel> {
        (crate::api::Model::read_prefix(&bytes)).map(|value| DecodedModel { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "read_prefix_with_limits")]
    #[pyo3(signature = (bytes, max_array_entries))]
    pub fn binding_read_prefix_with_limits(
        bytes: Vec<u8>,
        max_array_entries: usize,
    ) -> pyo3::PyResult<DecodedModel> {
        (crate::api::Model::read_prefix_with_limits(&bytes, max_array_entries))
            .map(|value| DecodedModel { inner: Some(value) })
    }
    #[pyo3(name = "write")]
    #[pyo3(signature = ())]
    pub fn binding_write(&self) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyBytes>> {
        ((self.inner.as_ref().ok_or_else(consumed)?).write()).map(|value| {
            let bytes = value;
            pyo3::Python::attach(|py| pyo3::types::PyBytes::new(py, &bytes).unbind())
        })
    }
    #[pyo3(name = "write_with_encoding")]
    #[pyo3(signature = (code))]
    pub fn binding_write_with_encoding(
        &self,
        code: u32,
    ) -> pyo3::PyResult<pyo3::Py<pyo3::types::PyBytes>> {
        ((self.inner.as_ref().ok_or_else(consumed)?).write_with_encoding(code)).map(|value| {
            let bytes = value;
            pyo3::Python::attach(|py| pyo3::types::PyBytes::new(py, &bytes).unbind())
        })
    }
}
#[pymethods]
impl Model {
    #[pyo3(name = "train")]
    #[pyo3(signature = (files, options))]
    pub fn binding_train(
        &self,
        files: pyo3::PyRef<'_, Datasets>,
        options: pyo3::PyRef<'_, TrainingOptions>,
    ) -> pyo3::PyResult<TrainingResult> {
        ((self.inner.as_ref().ok_or_else(consumed)?).train(
            files.inner.as_ref().ok_or_else(consumed)?,
            options.inner.as_ref().ok_or_else(consumed)?,
        ))
        .map(|value| TrainingResult { inner: Some(value) })
    }
    #[pyo3(name = "train_with_progress")]
    #[pyo3(signature = (files, options, progress))]
    pub fn binding_train_with_progress(
        &self,
        files: pyo3::PyRef<'_, Datasets>,
        options: pyo3::PyRef<'_, TrainingOptions>,
        progress: Py<PyAny>,
    ) -> pyo3::PyResult<TrainingResult> {
        ((self.inner.as_ref().ok_or_else(consumed)?).train_with_progress(
            files.inner.as_ref().ok_or_else(consumed)?,
            options.inner.as_ref().ok_or_else(consumed)?,
            &crate::callbacks::Function::new(progress)?,
        ))
        .map(|value| TrainingResult { inner: Some(value) })
    }
}
#[pymethods]
impl Model {
    #[pyo3(name = "untie")]
    #[pyo3(signature = (document))]
    pub fn binding_untie(
        &self,
        document: pyo3::PyRef<'_, SegmentationDocument>,
    ) -> pyo3::PyResult<UntiedModel> {
        ((self.inner.as_ref().ok_or_else(consumed)?)
            .untie(document.inner.as_ref().ok_or_else(consumed)?))
        .map(|value| UntiedModel { inner: Some(value) })
    }
}
