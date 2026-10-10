use super::*;
#[napi]
pub struct BatchOutputs {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::BatchOutputs>>,
}
#[napi]
impl BatchOutputs {
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
impl BatchOutputs {
    #[napi(getter, js_name = "raw")]
    pub fn binding_get_raw(&self) -> napi::Result<String> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .raw
            .clone())
    }
    #[napi(setter, js_name = "raw")]
    pub fn binding_set_raw(&self, value: String) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .raw = value;
        Ok(())
    }
    #[napi(getter, js_name = "parameters")]
    pub fn binding_get_parameters(&self) -> napi::Result<String> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .parameters
            .clone())
    }
    #[napi(setter, js_name = "parameters")]
    pub fn binding_set_parameters(&self, value: String) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .parameters = value;
        Ok(())
    }
    #[napi(getter, js_name = "mfcc")]
    pub fn binding_get_mfcc(&self) -> napi::Result<Option<String>> {
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
    pub fn binding_set_mfcc(&self, value: Option<String>) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .mfcc = value;
        Ok(())
    }
}
#[napi]
impl BatchOutputs {
    #[napi(constructor)]
    pub fn binding_new(
        raw: String,
        parameters: String,
        mfcc: Option<String>,
    ) -> napi::Result<Self> {
        Ok(BatchOutputs {
            inner: std::cell::RefCell::new(Some(crate::api::BatchOutputs::new(
                raw, parameters, mfcc,
            ))),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(BatchOutputs {
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
