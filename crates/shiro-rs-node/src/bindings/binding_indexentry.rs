use super::*;
#[napi]
pub struct IndexEntry {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::IndexEntry>>,
}
#[napi]
impl IndexEntry {
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
impl IndexEntry {
    #[napi(constructor)]
    pub fn binding_new(stem: String, phonemes_json: String) -> napi::Result<Self> {
        (crate::api::IndexEntry::new(&stem, &phonemes_json)).map(|value| IndexEntry {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(getter, js_name = "stem")]
    pub fn binding_stem(&self) -> napi::Result<String> {
        (self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .stem()
    }
    #[napi(setter, js_name = "stem")]
    pub fn binding_set_stem(&self, stem: String) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_stem(&stem);
        Ok(())
    }
    #[napi(js_name = "phonemes_json")]
    pub fn binding_phonemes_json(&self) -> napi::Result<String> {
        (self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .phonemes_json()
    }
    #[napi(js_name = "set_phonemes_json")]
    pub fn binding_set_phonemes_json(&self, json: String) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_phonemes_json(&json)
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(IndexEntry {
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
