use super::*;
#[napi]
pub struct FeatureFiles {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::FeatureFiles>>,
}
#[napi]
impl FeatureFiles {
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
impl FeatureFiles {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(FeatureFiles {
            inner: std::cell::RefCell::new(Some(crate::api::FeatureFiles::new())),
        })
    }
    #[napi(getter, js_name = "length")]
    pub fn binding_length(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .length() as f64)
    }
    #[napi(js_name = "names")]
    pub fn binding_names(&self) -> napi::Result<Vec<String>> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .names())
    }
    #[napi(js_name = "get")]
    pub fn binding_get(
        &self,
        filename: String,
    ) -> napi::Result<Option<napi::bindgen_prelude::Uint8Array>> {
        Ok(((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .get(&filename))
        .map(<napi::bindgen_prelude::Uint8Array>::from))
    }
    #[napi(js_name = "set")]
    pub fn binding_set(
        &self,
        filename: String,
        bytes: napi::bindgen_prelude::Uint8Array,
    ) -> napi::Result<()> {
        let storage_bytes = bytes.to_vec();
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set(&filename, &storage_bytes);
        Ok(())
    }
    #[napi(js_name = "remove")]
    pub fn binding_remove(&self, filename: String) -> napi::Result<bool> {
        Ok((self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .remove(&filename))
    }
    #[napi(js_name = "clear")]
    pub fn binding_clear(&self) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .clear();
        Ok(())
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(FeatureFiles {
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
