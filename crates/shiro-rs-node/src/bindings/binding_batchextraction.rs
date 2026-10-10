use super::*;
#[napi]
pub struct BatchExtraction {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::BatchExtraction>>,
}
#[napi]
impl BatchExtraction {
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
impl BatchExtraction {
    #[napi(constructor)]
    pub fn binding_new(
        audio: &Audio,
        features: &Features,
        outputs: &BatchOutputs,
    ) -> napi::Result<Self> {
        Ok(BatchExtraction {
            inner: std::cell::RefCell::new(Some(crate::api::BatchExtraction::new(
                audio
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                features
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                outputs
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(js_name = "audio")]
    pub fn binding_audio(&self) -> napi::Result<Audio> {
        Ok(Audio {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .audio(),
            )),
        })
    }
    #[napi(js_name = "set_audio")]
    pub fn binding_set_audio(&self, value: &Audio) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_audio(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "features")]
    pub fn binding_features(&self) -> napi::Result<Features> {
        Ok(Features {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .features(),
            )),
        })
    }
    #[napi(js_name = "set_features")]
    pub fn binding_set_features(&self, value: &Features) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_features(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "outputs")]
    pub fn binding_outputs(&self) -> napi::Result<BatchOutputs> {
        Ok(BatchOutputs {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .outputs(),
            )),
        })
    }
    #[napi(js_name = "set_outputs")]
    pub fn binding_set_outputs(&self, value: &BatchOutputs) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_outputs(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(BatchExtraction {
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
    #[napi(js_name = "extract_with_sequence")]
    pub fn binding_extract_with_sequence(
        wave: &Wave,
        stem: String,
        options: &BatchOptions,
        value: u32,
        sequence: &DitherSequence,
    ) -> napi::Result<Self> {
        (crate::api::BatchExtraction::extract_with_sequence(
            wave.inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            &stem,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            value,
            sequence
                .inner
                .try_borrow_mut()
                .map_err(borrowed)?
                .as_mut()
                .ok_or_else(consumed)?,
        ))
        .map(|value| BatchExtraction {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "extract")]
    pub fn binding_extract(
        wave: &Wave,
        stem: String,
        options: &BatchOptions,
        value: u32,
        #[napi(ts_arg_type = "() => number")] uniform: napi::bindgen_prelude::Unknown<'_>,
        env: napi::Env,
    ) -> napi::Result<Self> {
        (crate::api::BatchExtraction::extract(
            wave.inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            &stem,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            value,
            &crate::callbacks::Function::new(env, uniform)?,
        ))
        .map(|value| BatchExtraction {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
}
