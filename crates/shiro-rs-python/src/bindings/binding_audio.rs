use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Audio {
    pub(crate) inner: Option<crate::api::Audio>,
}
#[pymethods]
impl Audio {
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
impl Audio {
    #[new]
    #[pyo3(signature = (sample_rate, samples))]
    pub fn binding_new(sample_rate: u32, samples: Vec<f32>) -> pyo3::PyResult<Self> {
        Ok(Audio {
            inner: Some(crate::api::Audio::new(sample_rate, &samples)),
        })
    }
    #[getter(sample_rate)]
    pub fn binding_sample_rate(&self) -> pyo3::PyResult<u32> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).sample_rate())
    }
    #[setter(sample_rate)]
    pub fn binding_set_sample_rate(&mut self, value: u32) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_sample_rate(value);
        Ok(())
    }
    #[pyo3(name = "samples")]
    #[pyo3(signature = ())]
    pub fn binding_samples(&self) -> pyo3::PyResult<Vec<f32>> {
        Ok((self.inner.as_ref().ok_or_else(consumed)?).samples())
    }
    #[pyo3(name = "set_samples")]
    #[pyo3(signature = (samples))]
    pub fn binding_set_samples(&mut self, samples: Vec<f32>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_samples(&samples);
        Ok(())
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Audio {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
    #[staticmethod]
    #[pyo3(name = "prepare_with_sequence")]
    #[pyo3(signature = (wave, options, sequence))]
    pub fn binding_prepare_with_sequence(
        wave: pyo3::PyRef<'_, Wave>,
        options: pyo3::PyRef<'_, AudioOptions>,
        mut sequence: pyo3::PyRefMut<'_, DitherSequence>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Audio::prepare_with_sequence(
            wave.inner.as_ref().ok_or_else(consumed)?,
            options.inner.as_ref().ok_or_else(consumed)?,
            sequence.inner.as_mut().ok_or_else(consumed)?,
        ))
        .map(|value| Audio { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "prepare")]
    #[pyo3(signature = (wave, options, uniform))]
    pub fn binding_prepare(
        wave: pyo3::PyRef<'_, Wave>,
        options: pyo3::PyRef<'_, AudioOptions>,
        uniform: Py<PyAny>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::Audio::prepare(
            wave.inner.as_ref().ok_or_else(consumed)?,
            options.inner.as_ref().ok_or_else(consumed)?,
            &crate::callbacks::Function::new(uniform)?,
        ))
        .map(|value| Audio { inner: Some(value) })
    }
}
