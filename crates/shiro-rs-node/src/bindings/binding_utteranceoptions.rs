use super::*;
#[napi]
pub struct UtteranceOptions {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::UtteranceOptions>>,
}
#[napi]
impl UtteranceOptions {
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
impl UtteranceOptions {
    #[napi(getter, js_name = "utterances")]
    pub fn binding_get_utterances(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .utterances as f64)
    }
    #[napi(setter, js_name = "utterances")]
    pub fn binding_set_utterances(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .utterances = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "hop_seconds")]
    pub fn binding_get_hop_seconds(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .hop_seconds)
    }
    #[napi(setter, js_name = "hop_seconds")]
    pub fn binding_set_hop_seconds(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .hop_seconds = value;
        Ok(())
    }
    #[napi(getter, js_name = "minimum_silence_seconds")]
    pub fn binding_get_minimum_silence_seconds(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .minimum_silence_seconds)
    }
    #[napi(setter, js_name = "minimum_silence_seconds")]
    pub fn binding_set_minimum_silence_seconds(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .minimum_silence_seconds = value;
        Ok(())
    }
    #[napi(getter, js_name = "minimum_voicing_seconds")]
    pub fn binding_get_minimum_voicing_seconds(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .minimum_voicing_seconds)
    }
    #[napi(setter, js_name = "minimum_voicing_seconds")]
    pub fn binding_set_minimum_voicing_seconds(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .minimum_voicing_seconds = value;
        Ok(())
    }
    #[napi(getter, js_name = "iterations")]
    pub fn binding_get_iterations(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .iterations as f64)
    }
    #[napi(setter, js_name = "iterations")]
    pub fn binding_set_iterations(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .iterations = checked_usize(value)?;
        Ok(())
    }
}
#[napi]
impl UtteranceOptions {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(UtteranceOptions {
            inner: std::cell::RefCell::new(Some(crate::api::UtteranceOptions::new())),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(UtteranceOptions {
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
