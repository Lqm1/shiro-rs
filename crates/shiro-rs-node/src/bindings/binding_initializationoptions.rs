use super::*;
#[napi]
pub struct InitializationOptions {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::InitializationOptions>>,
}
#[napi]
impl InitializationOptions {
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
impl InitializationOptions {
    #[napi(getter, js_name = "flat_start")]
    pub fn binding_get_flat_start(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .flat_start)
    }
    #[napi(setter, js_name = "flat_start")]
    pub fn binding_set_flat_start(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .flat_start = value;
        Ok(())
    }
    #[napi(getter, js_name = "globally_tied")]
    pub fn binding_get_globally_tied(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .globally_tied)
    }
    #[napi(setter, js_name = "globally_tied")]
    pub fn binding_set_globally_tied(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .globally_tied = value;
        Ok(())
    }
    #[napi(getter, js_name = "variance_floor_ratio")]
    pub fn binding_get_variance_floor_ratio(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .variance_floor_ratio as f64)
    }
    #[napi(setter, js_name = "variance_floor_ratio")]
    pub fn binding_set_variance_floor_ratio(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .variance_floor_ratio = value as f32;
        Ok(())
    }
}
#[napi]
impl InitializationOptions {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(InitializationOptions {
            inner: std::cell::RefCell::new(Some(crate::api::InitializationOptions::new())),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(InitializationOptions {
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
