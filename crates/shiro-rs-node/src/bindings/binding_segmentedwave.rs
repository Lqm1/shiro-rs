use super::*;
#[napi]
pub struct SegmentedWave {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::SegmentedWave>>,
}
#[napi]
impl SegmentedWave {
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
impl SegmentedWave {
    #[napi(constructor)]
    pub fn binding_new(
        audio: &Audio,
        features: &Features,
        utterances: &SegmentedUtterances,
    ) -> napi::Result<Self> {
        Ok(SegmentedWave {
            inner: std::cell::RefCell::new(Some(crate::api::SegmentedWave::new(
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
                utterances
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(js_name = "split_with_sequence")]
    pub fn binding_split_with_sequence(
        wave: &Wave,
        filename: String,
        dimensions: f64,
        kind: u32,
        options: &UtteranceOptions,
        source: &UtteranceModelSource,
        sequence: &DitherSequence,
    ) -> napi::Result<Self> {
        (crate::api::SegmentedWave::split_with_sequence(
            wave.inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            &filename,
            checked_usize(dimensions)?,
            kind,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            source
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
        .map(|value| SegmentedWave {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "split")]
    #[allow(
        clippy::too_many_arguments,
        reason = "Keep the explicit cross-language split parameters and hidden N-API environment."
    )]
    pub fn binding_split(
        wave: &Wave,
        filename: String,
        dimensions: f64,
        kind: u32,
        options: &UtteranceOptions,
        source: &UtteranceModelSource,
        #[napi(ts_arg_type = "() => number")] uniform: napi::bindgen_prelude::Unknown<'_>,
        env: napi::Env,
    ) -> napi::Result<Self> {
        (crate::api::SegmentedWave::split(
            wave.inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            &filename,
            checked_usize(dimensions)?,
            kind,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            source
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            &crate::callbacks::Function::new(env, uniform)?,
        ))
        .map(|value| SegmentedWave {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(SegmentedWave {
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
#[napi]
impl SegmentedWave {
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
}
#[napi]
impl SegmentedWave {
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
}
#[napi]
impl SegmentedWave {
    #[napi(js_name = "utterances")]
    pub fn binding_utterances(&self) -> napi::Result<SegmentedUtterances> {
        Ok(SegmentedUtterances {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .utterances(),
            )),
        })
    }
    #[napi(js_name = "set_utterances")]
    pub fn binding_set_utterances(&self, value: &SegmentedUtterances) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_utterances(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
}
