use super::*;
#[napi]
pub struct SegmentationDocument {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::SegmentationDocument>>,
}
#[napi]
impl SegmentationDocument {
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
impl SegmentationDocument {
    #[napi(constructor)]
    pub fn binding_new(json: String) -> napi::Result<Self> {
        (crate::api::SegmentationDocument::new(&json)).map(|value| SegmentationDocument {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "json")]
    pub fn binding_json(&self) -> napi::Result<String> {
        (self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .json()
    }
    #[napi(js_name = "replace")]
    pub fn binding_replace(&self, json: String) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .replace(&json)
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(SegmentationDocument {
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
