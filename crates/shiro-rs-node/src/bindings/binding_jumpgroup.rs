use super::*;
#[napi]
pub struct JumpGroup {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::JumpGroup>>,
}
#[napi]
impl JumpGroup {
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
impl JumpGroup {
    #[napi(constructor)]
    pub fn binding_new(
        deltas: napi::bindgen_prelude::Int32Array,
        probabilities: napi::bindgen_prelude::Float32Array,
    ) -> napi::Result<Self> {
        let storage_deltas = deltas.to_vec();
        let storage_probabilities = probabilities.to_vec();
        (crate::api::JumpGroup::new(&storage_deltas, &storage_probabilities)).map(|value| {
            JumpGroup {
                inner: std::cell::RefCell::new(Some(value)),
            }
        })
    }
    #[napi(getter, js_name = "length")]
    pub fn binding_length(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .length() as f64)
    }
    #[napi(js_name = "deltas")]
    pub fn binding_deltas(&self) -> napi::Result<napi::bindgen_prelude::Int32Array> {
        Ok(<napi::bindgen_prelude::Int32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .deltas(),
        ))
    }
    #[napi(js_name = "probabilities")]
    pub fn binding_probabilities(&self) -> napi::Result<napi::bindgen_prelude::Float32Array> {
        Ok(<napi::bindgen_prelude::Float32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .probabilities(),
        ))
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(JumpGroup {
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
