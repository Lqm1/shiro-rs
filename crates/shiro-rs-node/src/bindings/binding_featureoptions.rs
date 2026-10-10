use super::*;
#[napi]
pub struct FeatureOptions {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::FeatureOptions>>,
}
#[napi]
impl FeatureOptions {
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
impl FeatureOptions {
    #[napi(getter, js_name = "kind")]
    pub fn binding_get_kind(&self) -> napi::Result<u32> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .kind)
    }
    #[napi(setter, js_name = "kind")]
    pub fn binding_set_kind(&self, value: u32) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .kind = value;
        Ok(())
    }
    #[napi(getter, js_name = "order")]
    pub fn binding_get_order(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .order as f64)
    }
    #[napi(setter, js_name = "order")]
    pub fn binding_set_order(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .order = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "channels")]
    pub fn binding_get_channels(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .channels as f64)
    }
    #[napi(setter, js_name = "channels")]
    pub fn binding_set_channels(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .channels = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "frame_length")]
    pub fn binding_get_frame_length(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .frame_length as f64)
    }
    #[napi(setter, js_name = "frame_length")]
    pub fn binding_set_frame_length(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .frame_length = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "hop")]
    pub fn binding_get_hop(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .hop as f64)
    }
    #[napi(setter, js_name = "hop")]
    pub fn binding_set_hop(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .hop = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "sample_rate_hz")]
    pub fn binding_get_sample_rate_hz(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .sample_rate_hz as f64)
    }
    #[napi(setter, js_name = "sample_rate_hz")]
    pub fn binding_set_sample_rate_hz(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .sample_rate_hz = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "minimum_bandwidth_hz")]
    pub fn binding_get_minimum_bandwidth_hz(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .minimum_bandwidth_hz as f64)
    }
    #[napi(setter, js_name = "minimum_bandwidth_hz")]
    pub fn binding_set_minimum_bandwidth_hz(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .minimum_bandwidth_hz = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "warp")]
    pub fn binding_get_warp(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .warp as f64)
    }
    #[napi(setter, js_name = "warp")]
    pub fn binding_set_warp(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .warp = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "include_dc")]
    pub fn binding_get_include_dc(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .include_dc)
    }
    #[napi(setter, js_name = "include_dc")]
    pub fn binding_set_include_dc(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .include_dc = value;
        Ok(())
    }
    #[napi(getter, js_name = "energy")]
    pub fn binding_get_energy(&self) -> napi::Result<u32> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .energy)
    }
    #[napi(setter, js_name = "energy")]
    pub fn binding_set_energy(&self, value: u32) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .energy = value;
        Ok(())
    }
    #[napi(getter, js_name = "delta")]
    pub fn binding_get_delta(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .delta)
    }
    #[napi(setter, js_name = "delta")]
    pub fn binding_set_delta(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .delta = value;
        Ok(())
    }
    #[napi(getter, js_name = "acceleration")]
    pub fn binding_get_acceleration(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .acceleration)
    }
    #[napi(setter, js_name = "acceleration")]
    pub fn binding_set_acceleration(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .acceleration = value;
        Ok(())
    }
}
#[napi]
impl FeatureOptions {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(FeatureOptions {
            inner: std::cell::RefCell::new(Some(crate::api::FeatureOptions::new())),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(FeatureOptions {
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
