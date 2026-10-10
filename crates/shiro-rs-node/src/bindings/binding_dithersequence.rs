use super::*;
#[napi]
pub struct DitherSequence {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::DitherSequence>>,
}
#[napi]
impl DitherSequence {
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
impl DitherSequence {
    #[napi(js_name = "windows")]
    pub fn binding_windows() -> napi::Result<Self> {
        Ok(DitherSequence {
            inner: std::cell::RefCell::new(Some(crate::api::DitherSequence::windows())),
        })
    }
    #[napi(js_name = "linux_gnu")]
    pub fn binding_linux_gnu() -> napi::Result<Self> {
        Ok(DitherSequence {
            inner: std::cell::RefCell::new(Some(crate::api::DitherSequence::linux_gnu())),
        })
    }
    #[napi(js_name = "next_uniform")]
    pub fn binding_next_uniform(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .next_uniform() as f64)
    }
}
