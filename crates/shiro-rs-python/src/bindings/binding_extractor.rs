use super::*;
#[pyclass(module = "shiro_rs")]
pub struct Extractor {
    pub(crate) inner: Option<crate::api::Extractor>,
}
#[pymethods]
impl Extractor {
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
impl Extractor {
    #[staticmethod]
    #[pyo3(name = "native")]
    #[pyo3(signature = (preset))]
    pub fn binding_native(preset: u32) -> pyo3::PyResult<Self> {
        (crate::api::Extractor::native(preset)).map(|value| Extractor { inner: Some(value) })
    }
    #[staticmethod]
    #[pyo3(name = "sptk")]
    #[pyo3(signature = (programs))]
    pub fn binding_sptk(programs: pyo3::PyRef<'_, SptkPrograms>) -> pyo3::PyResult<Self> {
        Ok(Extractor {
            inner: Some(crate::api::Extractor::sptk(
                programs.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
    #[staticmethod]
    #[pyo3(name = "lua")]
    #[pyo3(signature = (interpreter, script, executable_directory))]
    pub fn binding_lua(
        interpreter: String,
        script: String,
        executable_directory: String,
    ) -> pyo3::PyResult<Self> {
        Ok(Extractor {
            inner: Some(crate::api::Extractor::lua(
                interpreter,
                script,
                executable_directory,
            )),
        })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(Extractor {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
}
