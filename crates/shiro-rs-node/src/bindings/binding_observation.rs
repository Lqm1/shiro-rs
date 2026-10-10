use super::*;
#[napi]
pub struct Observation {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Observation>>,
}
#[napi]
impl Observation {
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
impl Observation {
    #[napi(constructor)]
    pub fn binding_new(
        frames: f64,
        dimensions: napi::bindgen_prelude::Uint32Array,
    ) -> napi::Result<Self> {
        let storage_dimensions = dimensions.to_vec();
        (crate::api::Observation::new(checked_usize(frames)?, &storage_dimensions)).map(|value| {
            Observation {
                inner: std::cell::RefCell::new(Some(value)),
            }
        })
    }
    #[napi(js_name = "read_rawfloat")]
    pub fn binding_read_rawfloat(
        bytes: napi::bindgen_prelude::Uint8Array,
        dimensions: napi::bindgen_prelude::Uint32Array,
        maximum_frames: f64,
    ) -> napi::Result<Self> {
        let storage_bytes = bytes.to_vec();
        let storage_dimensions = dimensions.to_vec();
        (crate::api::Observation::read_rawfloat(
            &storage_bytes,
            &storage_dimensions,
            checked_usize(maximum_frames)?,
        ))
        .map(|value| Observation {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "from_model_rawfloat")]
    pub fn binding_from_model_rawfloat(
        bytes: napi::bindgen_prelude::Uint8Array,
        model: &Model,
        maximum_frames: f64,
    ) -> napi::Result<Self> {
        let storage_bytes = bytes.to_vec();
        (crate::api::Observation::from_model_rawfloat(
            &storage_bytes,
            model
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            checked_usize(maximum_frames)?,
        ))
        .map(|value| Observation {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(getter, js_name = "frames")]
    pub fn binding_frames(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .frames() as f64)
    }
    #[napi(getter, js_name = "streams")]
    pub fn binding_streams(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .streams() as f64)
    }
    #[napi(js_name = "stream")]
    pub fn binding_stream(&self, index: f64) -> napi::Result<Option<ObservationStream>> {
        Ok(((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .stream(checked_usize(index)?))
        .map(|value| ObservationStream {
            inner: std::cell::RefCell::new(Some(value)),
        }))
    }
    #[napi(js_name = "set_stream")]
    pub fn binding_set_stream(&self, index: f64, value: &ObservationStream) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_stream(
            checked_usize(index)?,
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        )
    }
    #[napi(js_name = "replace")]
    pub fn binding_replace(&self, frames: f64, streams: &ObservationStreams) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .replace(
            checked_usize(frames)?,
            streams
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        )
    }
    #[napi(js_name = "frame")]
    pub fn binding_frame(
        &self,
        stream: f64,
        time: f64,
    ) -> napi::Result<Option<napi::bindgen_prelude::Float32Array>> {
        Ok(((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .frame(checked_usize(stream)?, checked_usize(time)?))
        .map(<napi::bindgen_prelude::Float32Array>::from))
    }
    #[napi(js_name = "validate")]
    pub fn binding_validate(&self) -> napi::Result<()> {
        (self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .validate()
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(Observation {
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
    #[napi(js_name = "write")]
    pub fn binding_write(&self) -> napi::Result<napi::bindgen_prelude::Uint8Array> {
        ((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .write())
        .map(<napi::bindgen_prelude::Uint8Array>::from)
    }
}
