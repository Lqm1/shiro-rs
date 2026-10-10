use super::*;
#[napi]
pub struct UtteranceModelSource {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::UtteranceModelSource>>,
}
#[napi]
impl UtteranceModelSource {
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
impl UtteranceModelSource {
    #[napi(js_name = "fresh")]
    pub fn binding_fresh() -> napi::Result<Self> {
        Ok(UtteranceModelSource {
            inner: std::cell::RefCell::new(Some(crate::api::UtteranceModelSource::fresh())),
        })
    }
    #[napi(js_name = "initialized")]
    pub fn binding_initialized(model: &Model) -> napi::Result<Self> {
        Ok(UtteranceModelSource {
            inner: std::cell::RefCell::new(Some(crate::api::UtteranceModelSource::initialized(
                model
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(js_name = "trained")]
    pub fn binding_trained(model: &Model) -> napi::Result<Self> {
        Ok(UtteranceModelSource {
            inner: std::cell::RefCell::new(Some(crate::api::UtteranceModelSource::trained(
                model
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(getter, js_name = "kind")]
    pub fn binding_kind(&self) -> napi::Result<u32> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .kind())
    }
    #[napi(js_name = "model")]
    pub fn binding_model(&self) -> napi::Result<Option<Model>> {
        Ok(((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .model())
        .map(|value| Model {
            inner: std::cell::RefCell::new(Some(value)),
        }))
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(UtteranceModelSource {
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
