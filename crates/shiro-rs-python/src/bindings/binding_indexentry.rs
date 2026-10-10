use super::*;
#[pyclass(module = "shiro_rs")]
pub struct IndexEntry {
    pub(crate) inner: Option<crate::api::IndexEntry>,
}
#[pymethods]
impl IndexEntry {
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
impl IndexEntry {
    #[new]
    #[pyo3(signature = (stem, phonemes_json))]
    pub fn binding_new(stem: &str, phonemes_json: &str) -> pyo3::PyResult<Self> {
        (crate::api::IndexEntry::new(stem, phonemes_json))
            .map(|value| IndexEntry { inner: Some(value) })
    }
    #[getter(stem)]
    pub fn binding_stem(&self) -> pyo3::PyResult<String> {
        (self.inner.as_ref().ok_or_else(consumed)?).stem()
    }
    #[setter(stem)]
    pub fn binding_set_stem(&mut self, stem: &str) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).set_stem(stem);
        Ok(())
    }
    #[pyo3(name = "phonemes_json")]
    #[pyo3(signature = ())]
    pub fn binding_phonemes_json(&self) -> pyo3::PyResult<String> {
        (self.inner.as_ref().ok_or_else(consumed)?).phonemes_json()
    }
    #[pyo3(name = "set_phonemes_json")]
    #[pyo3(signature = (json))]
    pub fn binding_set_phonemes_json(&mut self, json: &str) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).set_phonemes_json(json)
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(IndexEntry {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
