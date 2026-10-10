use super::*;
#[pyclass(module = "shiro_rs")]
pub struct BatchExtraction {
    pub(crate) inner: Option<crate::api::BatchExtraction>,
}
#[pymethods]
impl BatchExtraction {
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
impl BatchExtraction {
    #[new]
    #[pyo3(signature = (audio, features, outputs))]
    pub fn binding_new(
        audio: pyo3::PyRef<'_, Audio>,
        features: pyo3::PyRef<'_, Features>,
        outputs: pyo3::PyRef<'_, BatchOutputs>,
    ) -> pyo3::PyResult<Self> {
        Ok(BatchExtraction {
            inner: Some(crate::api::BatchExtraction::new(
                audio.inner.as_ref().ok_or_else(consumed)?,
                features.inner.as_ref().ok_or_else(consumed)?,
                outputs.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
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
    #[pyo3(name = "outputs")]
    #[pyo3(signature = ())]
    pub fn binding_outputs(&self) -> pyo3::PyResult<BatchOutputs> {
        Ok(BatchOutputs {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).outputs()),
        })
    }
    #[pyo3(name = "set_outputs")]
    #[pyo3(signature = (value))]
    pub fn binding_set_outputs(
        &mut self,
        value: pyo3::PyRef<'_, BatchOutputs>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_outputs(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(BatchExtraction {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
    #[staticmethod]
    #[pyo3(name = "extract_with_sequence")]
    #[pyo3(signature = (wave, stem, options, value, sequence))]
    pub fn binding_extract_with_sequence(
        wave: pyo3::PyRef<'_, Wave>,
        stem: &str,
        options: pyo3::PyRef<'_, BatchOptions>,
        value: u32,
        mut sequence: pyo3::PyRefMut<'_, DitherSequence>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::BatchExtraction::extract_with_sequence(
            wave.inner.as_ref().ok_or_else(consumed)?,
            stem,
            options.inner.as_ref().ok_or_else(consumed)?,
            value,
            sequence.inner.as_mut().ok_or_else(consumed)?,
        ))
        .map(|value| BatchExtraction { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "extract")]
    #[pyo3(signature = (wave, stem, options, value, uniform))]
    pub fn binding_extract(
        wave: pyo3::PyRef<'_, Wave>,
        stem: &str,
        options: pyo3::PyRef<'_, BatchOptions>,
        value: u32,
        uniform: Py<PyAny>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::BatchExtraction::extract(
            wave.inner.as_ref().ok_or_else(consumed)?,
            stem,
            options.inner.as_ref().ok_or_else(consumed)?,
            value,
            &crate::callbacks::Function::new(uniform)?,
        ))
        .map(|value| BatchExtraction { inner: Some(value) })
    }
}
