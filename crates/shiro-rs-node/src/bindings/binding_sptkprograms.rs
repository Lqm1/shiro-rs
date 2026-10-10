use super::*;
#[napi]
pub struct SptkPrograms {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::SptkPrograms>>,
}
#[napi]
impl SptkPrograms {
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
impl SptkPrograms {
    #[napi(getter, js_name = "frame")]
    pub fn binding_get_frame(&self) -> napi::Result<String> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .frame
            .clone())
    }
    #[napi(setter, js_name = "frame")]
    pub fn binding_set_frame(&self, value: String) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .frame = value;
        Ok(())
    }
    #[napi(getter, js_name = "mfcc")]
    pub fn binding_get_mfcc(&self) -> napi::Result<String> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .mfcc
            .clone())
    }
    #[napi(setter, js_name = "mfcc")]
    pub fn binding_set_mfcc(&self, value: String) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .mfcc = value;
        Ok(())
    }
    #[napi(getter, js_name = "delta")]
    pub fn binding_get_delta(&self) -> napi::Result<String> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .delta
            .clone())
    }
    #[napi(setter, js_name = "delta")]
    pub fn binding_set_delta(&self, value: String) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .delta = value;
        Ok(())
    }
}
#[napi]
impl SptkPrograms {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(SptkPrograms {
            inner: std::cell::RefCell::new(Some(crate::api::SptkPrograms::new())),
        })
    }
}
