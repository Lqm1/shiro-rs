use super::*;
#[napi]
pub struct DecodedModel {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::DecodedModel>>,
}
#[napi]
impl DecodedModel {
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
impl DecodedModel {
    #[napi(getter, js_name = "position")]
    pub fn binding_position(&self) -> napi::Result<napi::bindgen_prelude::BigInt> {
        Ok(napi::bindgen_prelude::BigInt::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .position(),
        ))
    }
    #[napi(js_name = "parameters")]
    pub fn binding_parameters(&self) -> napi::Result<Model> {
        Ok(Model {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .parameters(),
            )),
        })
    }
    #[napi(js_name = "into_parameters")]
    pub fn binding_into_parameters(&self) -> napi::Result<Model> {
        Ok(Model {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow_mut()
                    .map_err(borrowed)?
                    .take()
                    .ok_or_else(consumed)?)
                .into_parameters(),
            )),
        })
    }
}
