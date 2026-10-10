use super::*;
#[napi]
pub struct Label {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Label>>,
}
#[napi]
impl Label {
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
impl Label {
    #[napi(getter, js_name = "start")]
    pub fn binding_get_start(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .start)
    }
    #[napi(setter, js_name = "start")]
    pub fn binding_set_start(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .start = value;
        Ok(())
    }
    #[napi(getter, js_name = "end")]
    pub fn binding_get_end(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .end)
    }
    #[napi(setter, js_name = "end")]
    pub fn binding_set_end(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .end = value;
        Ok(())
    }
}
#[napi]
impl Label {
    #[napi(constructor)]
    pub fn binding_new(start: f64, end: f64, name: String) -> napi::Result<Self> {
        Ok(Label {
            inner: std::cell::RefCell::new(Some(crate::api::Label::new(start, end, name))),
        })
    }
    #[napi(getter, js_name = "name")]
    pub fn binding_name(&self) -> napi::Result<String> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .name())
    }
    #[napi(setter, js_name = "name")]
    pub fn binding_set_name(&self, name: String) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_name(name);
        Ok(())
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(Label {
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
