use super::*;
#[napi]
pub struct Audio {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Audio>>,
}
#[napi]
impl Audio {
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
impl Audio {
    #[napi(constructor)]
    pub fn binding_new(
        sample_rate: u32,
        samples: napi::bindgen_prelude::Float32Array,
    ) -> napi::Result<Self> {
        let storage_samples = samples.to_vec();
        Ok(Audio {
            inner: std::cell::RefCell::new(Some(crate::api::Audio::new(
                sample_rate,
                &storage_samples,
            ))),
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
        Ok(Audio {
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
    #[napi(js_name = "prepare_with_sequence")]
    pub fn binding_prepare_with_sequence(
        wave: &Wave,
        options: &AudioOptions,
        sequence: &DitherSequence,
    ) -> napi::Result<Self> {
        (crate::api::Audio::prepare_with_sequence(
            wave.inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            sequence
                .inner
                .try_borrow_mut()
                .map_err(borrowed)?
                .as_mut()
                .ok_or_else(consumed)?,
        ))
        .map(|value| Audio {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "prepare")]
    pub fn binding_prepare(
        wave: &Wave,
        options: &AudioOptions,
        #[napi(ts_arg_type = "() => number")] uniform: napi::bindgen_prelude::Unknown<'_>,
        env: napi::Env,
    ) -> napi::Result<Self> {
        (crate::api::Audio::prepare(
            wave.inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            &crate::callbacks::Function::new(env, uniform)?,
        ))
        .map(|value| Audio {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
}
