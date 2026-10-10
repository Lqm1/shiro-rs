use super::*;
#[napi]
pub struct AudioOptions {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::AudioOptions>>,
}
#[napi]
impl AudioOptions {
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
impl AudioOptions {
    #[napi(getter, js_name = "normalize")]
    pub fn binding_get_normalize(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .normalize)
    }
    #[napi(setter, js_name = "normalize")]
    pub fn binding_set_normalize(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .normalize = value;
        Ok(())
    }
    #[napi(getter, js_name = "dither_level")]
    pub fn binding_get_dither_level(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .dither_level as f64)
    }
    #[napi(setter, js_name = "dither_level")]
    pub fn binding_set_dither_level(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .dither_level = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "output_sample_rate")]
    pub fn binding_get_output_sample_rate(&self) -> napi::Result<Option<u32>> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .output_sample_rate)
    }
    #[napi(setter, js_name = "output_sample_rate")]
    pub fn binding_set_output_sample_rate(&self, value: Option<u32>) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .output_sample_rate = value;
        Ok(())
    }
    #[napi(getter, js_name = "boundary")]
    pub fn binding_get_boundary(&self) -> napi::Result<u32> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .boundary)
    }
    #[napi(setter, js_name = "boundary")]
    pub fn binding_set_boundary(&self, value: u32) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .boundary = value;
        Ok(())
    }
    #[napi(getter, js_name = "kernel")]
    pub fn binding_get_kernel(&self) -> napi::Result<u32> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .kernel)
    }
    #[napi(setter, js_name = "kernel")]
    pub fn binding_set_kernel(&self, value: u32) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .kernel = value;
        Ok(())
    }
}
#[napi]
impl AudioOptions {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(AudioOptions {
            inner: std::cell::RefCell::new(Some(crate::api::AudioOptions::new())),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(AudioOptions {
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
