use super::*;
#[napi]
pub struct Segmentation {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Segmentation>>,
}
#[napi]
impl Segmentation {
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
impl Segmentation {
    #[napi(constructor)]
    pub fn binding_new(streams: f64, segments: f64) -> napi::Result<Self> {
        (crate::api::Segmentation::new(checked_usize(streams)?, checked_usize(segments)?)).map(
            |value| Segmentation {
                inner: std::cell::RefCell::new(Some(value)),
            },
        )
    }
    #[napi(js_name = "from_states")]
    pub fn binding_from_states(states: &States, model: &Model) -> napi::Result<Self> {
        (crate::api::Segmentation::from_states(
            states
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            model
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        ))
        .map(|value| Segmentation {
            inner: std::cell::RefCell::new(Some(value)),
        })
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
    #[napi(getter, js_name = "segments")]
    pub fn binding_segments(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .segments() as f64)
    }
    #[napi(js_name = "boundaries")]
    pub fn binding_boundaries(&self) -> napi::Result<napi::bindgen_prelude::Int32Array> {
        Ok(<napi::bindgen_prelude::Int32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .boundaries(),
        ))
    }
    #[napi(js_name = "duration_states")]
    pub fn binding_duration_states(&self) -> napi::Result<napi::bindgen_prelude::Int32Array> {
        Ok(<napi::bindgen_prelude::Int32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .duration_states(),
        ))
    }
    #[napi(js_name = "output_states")]
    pub fn binding_output_states(
        &self,
        stream: f64,
    ) -> napi::Result<napi::bindgen_prelude::Int32Array> {
        ((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .output_states(checked_usize(stream)?))
        .map(<napi::bindgen_prelude::Int32Array>::from)
    }
    #[napi(js_name = "set_boundaries")]
    pub fn binding_set_boundaries(
        &self,
        values: napi::bindgen_prelude::Int32Array,
    ) -> napi::Result<()> {
        let storage_values = values.to_vec();
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_boundaries(&storage_values)
    }
    #[napi(js_name = "set_duration_states")]
    pub fn binding_set_duration_states(
        &self,
        values: napi::bindgen_prelude::Int32Array,
    ) -> napi::Result<()> {
        let storage_values = values.to_vec();
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_duration_states(&storage_values)
    }
    #[napi(js_name = "set_output_states")]
    pub fn binding_set_output_states(
        &self,
        stream: f64,
        values: napi::bindgen_prelude::Int32Array,
    ) -> napi::Result<()> {
        let storage_values = values.to_vec();
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_output_states(checked_usize(stream)?, &storage_values)
    }
    #[napi(js_name = "outgoing")]
    pub fn binding_outgoing(&self, state: f64) -> napi::Result<Option<JumpGroup>> {
        Ok(((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .outgoing(checked_usize(state)?))
        .map(|value| JumpGroup {
            inner: std::cell::RefCell::new(Some(value)),
        }))
    }
    #[napi(js_name = "set_outgoing")]
    pub fn binding_set_outgoing(&self, state: f64, jumps: &JumpGroup) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_outgoing(
            checked_usize(state)?,
            jumps
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        )
    }
    #[napi(js_name = "replace")]
    pub fn binding_replace(&self, value: &Segmentation) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .replace(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
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
        Ok(Segmentation {
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
