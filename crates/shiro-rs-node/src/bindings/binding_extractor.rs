use super::*;
#[napi]
pub struct Extractor {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Extractor>>,
}
#[napi]
impl Extractor {
    #[napi]
    pub fn close(&self) -> napi::Result<()> {
        *self.inner.try_borrow_mut().map_err(borrowed)? = None;
        Ok(())
    }
    #[napi]
    pub fn free(&self) -> napi::Result<()> {
        self.close()
    }
    #[napi(getter, js_name = "is_closed")]
    pub fn is_closed(&self) -> napi::Result<bool> {
        Ok(self.inner.try_borrow().map_err(borrowed)?.is_none())
    }
}
#[napi]
impl Extractor {
    #[napi(js_name = "native")]
    pub fn binding_native(preset: u32) -> napi::Result<Self> {
        (crate::api::Extractor::native(preset)).map(|value| Extractor {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "sptk")]
    pub fn binding_sptk(programs: &SptkPrograms) -> napi::Result<Self> {
        Ok(Extractor {
            inner: std::cell::RefCell::new(Some(crate::api::Extractor::sptk(
                programs
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(js_name = "lua")]
    pub fn binding_lua(
        interpreter: String,
        script: String,
        executable_directory: String,
    ) -> napi::Result<Self> {
        Ok(Extractor {
            inner: std::cell::RefCell::new(Some(crate::api::Extractor::lua(
                interpreter,
                script,
                executable_directory,
            ))),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(Extractor {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .cloned(),
            )),
        })
    }
}
