use super::*;
#[napi]
pub struct PhoneMapOptions {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::PhoneMapOptions>>,
}
#[napi]
impl PhoneMapOptions {
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
impl PhoneMapOptions {
    #[napi(getter, js_name = "states_per_phone")]
    pub fn binding_get_states_per_phone(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .states_per_phone as f64)
    }
    #[napi(setter, js_name = "states_per_phone")]
    pub fn binding_set_states_per_phone(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .states_per_phone = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "streams")]
    pub fn binding_get_streams(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .streams as f64)
    }
    #[napi(setter, js_name = "streams")]
    pub fn binding_set_streams(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .streams = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "topology")]
    pub fn binding_get_topology(&self) -> napi::Result<Option<String>> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .topology
            .clone())
    }
    #[napi(setter, js_name = "topology")]
    pub fn binding_set_topology(&self, value: Option<String>) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .topology = value;
        Ok(())
    }
    #[napi(getter, js_name = "weak_skips")]
    pub fn binding_get_weak_skips(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .weak_skips)
    }
    #[napi(setter, js_name = "weak_skips")]
    pub fn binding_set_weak_skips(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .weak_skips = value;
        Ok(())
    }
}
#[napi]
impl PhoneMapOptions {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(PhoneMapOptions {
            inner: std::cell::RefCell::new(Some(crate::api::PhoneMapOptions::new())),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(PhoneMapOptions {
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
