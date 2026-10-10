use super::*;
#[napi]
pub struct Assignment {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Assignment>>,
}
#[napi]
impl Assignment {
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
impl Assignment {
    #[napi(getter, js_name = "state")]
    pub fn binding_get_state(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .state as f64)
    }
    #[napi(setter, js_name = "state")]
    pub fn binding_set_state(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .state = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "file")]
    pub fn binding_get_file(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .file as f64)
    }
    #[napi(setter, js_name = "file")]
    pub fn binding_set_file(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .file = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "segment")]
    pub fn binding_get_segment(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .segment as f64)
    }
    #[napi(setter, js_name = "segment")]
    pub fn binding_set_segment(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .segment = checked_usize(value)?;
        Ok(())
    }
}
#[napi]
impl Assignment {
    #[napi(constructor)]
    pub fn binding_new(state: f64, file: f64, segment: f64) -> napi::Result<Self> {
        Ok(Assignment {
            inner: std::cell::RefCell::new(Some(crate::api::Assignment::new(
                checked_usize(state)?,
                checked_usize(file)?,
                checked_usize(segment)?,
            ))),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(Assignment {
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
