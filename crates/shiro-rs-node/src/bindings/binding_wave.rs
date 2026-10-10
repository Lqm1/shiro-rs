use super::*;
#[napi]
pub struct Wave {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Wave>>,
}
#[napi]
impl Wave {
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
impl Wave {
    #[napi(constructor)]
    pub fn binding_new(
        sample_rate: u32,
        bits_per_sample: u16,
        channels: u16,
        encoding: u32,
        samples: napi::bindgen_prelude::Float32Array,
    ) -> napi::Result<Self> {
        let storage_samples = samples.to_vec();
        (crate::api::Wave::new(
            sample_rate,
            bits_per_sample,
            channels,
            encoding,
            &storage_samples,
        ))
        .map(|value| Wave {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "read")]
    pub fn binding_read(
        bytes: napi::bindgen_prelude::Uint8Array,
        maximum_frames: f64,
    ) -> napi::Result<Self> {
        let storage_bytes = bytes.to_vec();
        (crate::api::Wave::read(&storage_bytes, checked_usize(maximum_frames)?)).map(|value| Wave {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(getter, js_name = "sample_rate")]
    pub fn binding_sample_rate(&self) -> napi::Result<u32> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .sample_rate())
    }
    #[napi(setter, js_name = "sample_rate")]
    pub fn binding_set_sample_rate(&self, value: u32) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_sample_rate(value);
        Ok(())
    }
    #[napi(getter, js_name = "bits_per_sample")]
    pub fn binding_bits_per_sample(&self) -> napi::Result<u16> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .bits_per_sample())
    }
    #[napi(setter, js_name = "bits_per_sample")]
    pub fn binding_set_bits_per_sample(&self, value: u16) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_bits_per_sample(value);
        Ok(())
    }
    #[napi(getter, js_name = "channels")]
    pub fn binding_channels(&self) -> napi::Result<u16> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .channels())
    }
    #[napi(setter, js_name = "channels")]
    pub fn binding_set_channels(&self, value: u16) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_channels(value);
        Ok(())
    }
    #[napi(getter, js_name = "encoding")]
    pub fn binding_encoding(&self) -> napi::Result<u32> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .encoding())
    }
    #[napi(setter, js_name = "encoding")]
    pub fn binding_set_encoding(&self, value: u32) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_encoding(value)
    }
    #[napi(js_name = "samples")]
    pub fn binding_samples(&self) -> napi::Result<napi::bindgen_prelude::Float32Array> {
        Ok(<napi::bindgen_prelude::Float32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .samples(),
        ))
    }
    #[napi(js_name = "set_samples")]
    pub fn binding_set_samples(
        &self,
        samples: napi::bindgen_prelude::Float32Array,
    ) -> napi::Result<()> {
        let storage_samples = samples.to_vec();
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_samples(&storage_samples);
        Ok(())
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(Wave {
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
