use super::*;
#[pyclass(module = "shiro_rs")]
pub struct SegmentedWave {
    pub(crate) inner: Option<crate::api::SegmentedWave>,
}
#[pymethods]
impl SegmentedWave {
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
impl SegmentedWave {
    #[new]
    #[pyo3(signature = (audio, features, utterances))]
    pub fn binding_new(
        audio: pyo3::PyRef<'_, Audio>,
        features: pyo3::PyRef<'_, Features>,
        utterances: pyo3::PyRef<'_, SegmentedUtterances>,
    ) -> pyo3::PyResult<Self> {
        Ok(SegmentedWave {
            inner: Some(crate::api::SegmentedWave::new(
                audio.inner.as_ref().ok_or_else(consumed)?,
                features.inner.as_ref().ok_or_else(consumed)?,
                utterances.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
    #[staticmethod]
    #[pyo3(name = "split_with_sequence")]
    #[pyo3(signature = (wave, filename, dimensions, kind, options, source, sequence))]
    pub fn binding_split_with_sequence(
        wave: pyo3::PyRef<'_, Wave>,
        filename: &str,
        dimensions: usize,
        kind: u32,
        options: pyo3::PyRef<'_, UtteranceOptions>,
        source: pyo3::PyRef<'_, UtteranceModelSource>,
        mut sequence: pyo3::PyRefMut<'_, DitherSequence>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::SegmentedWave::split_with_sequence(
            wave.inner.as_ref().ok_or_else(consumed)?,
            filename,
            dimensions,
            kind,
            options.inner.as_ref().ok_or_else(consumed)?,
            source.inner.as_ref().ok_or_else(consumed)?,
            sequence.inner.as_mut().ok_or_else(consumed)?,
        ))
        .map(|value| SegmentedWave { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "split")]
    #[pyo3(signature = (wave, filename, dimensions, kind, options, source, uniform))]
    pub fn binding_split(
        wave: pyo3::PyRef<'_, Wave>,
        filename: &str,
        dimensions: usize,
        kind: u32,
        options: pyo3::PyRef<'_, UtteranceOptions>,
        source: pyo3::PyRef<'_, UtteranceModelSource>,
        uniform: Py<PyAny>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::SegmentedWave::split(
            wave.inner.as_ref().ok_or_else(consumed)?,
            filename,
            dimensions,
            kind,
            options.inner.as_ref().ok_or_else(consumed)?,
            source.inner.as_ref().ok_or_else(consumed)?,
            &crate::callbacks::Function::new(uniform)?,
        ))
        .map(|value| SegmentedWave { inner: Some(value) })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(SegmentedWave {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
#[pymethods]
impl SegmentedWave {
    #[pyo3(name = "audio")]
    #[pyo3(signature = ())]
    pub fn binding_audio(&self) -> pyo3::PyResult<Audio> {
        Ok(Audio {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).audio()),
        })
    }
    #[pyo3(name = "set_audio")]
    #[pyo3(signature = (value))]
    pub fn binding_set_audio(&mut self, value: pyo3::PyRef<'_, Audio>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_audio(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
#[pymethods]
impl SegmentedWave {
    #[pyo3(name = "features")]
    #[pyo3(signature = ())]
    pub fn binding_features(&self) -> pyo3::PyResult<Features> {
        Ok(Features {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).features()),
        })
    }
    #[pyo3(name = "set_features")]
    #[pyo3(signature = (value))]
    pub fn binding_set_features(&mut self, value: pyo3::PyRef<'_, Features>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_features(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
#[pymethods]
impl SegmentedWave {
    #[pyo3(name = "utterances")]
    #[pyo3(signature = ())]
    pub fn binding_utterances(&self) -> pyo3::PyResult<SegmentedUtterances> {
        Ok(SegmentedUtterances {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).utterances()),
        })
    }
    #[pyo3(name = "set_utterances")]
    #[pyo3(signature = (value))]
    pub fn binding_set_utterances(
        &mut self,
        value: pyo3::PyRef<'_, SegmentedUtterances>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_utterances(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
