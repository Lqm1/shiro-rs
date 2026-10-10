use super::*;
#[napi]
pub struct Features {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Features>>,
}
#[napi]
impl Features {
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
impl Features {
    #[napi(constructor)]
    pub fn binding_new(
        frames: f64,
        columns: f64,
        values: napi::bindgen_prelude::Float32Array,
    ) -> napi::Result<Self> {
        let storage_values = values.to_vec();
        Ok(Features {
            inner: std::cell::RefCell::new(Some(crate::api::Features::new(
                checked_usize(frames)?,
                checked_usize(columns)?,
                &storage_values,
            ))),
        })
    }
    #[napi(js_name = "extract")]
    pub fn binding_extract(
        signal: napi::bindgen_prelude::Float32Array,
        options: &FeatureOptions,
    ) -> napi::Result<Self> {
        let storage_signal = signal.to_vec();
        (crate::api::Features::extract(
            &storage_signal,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        ))
        .map(|value| Features {
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
    #[napi(setter, js_name = "frames")]
    pub fn binding_set_frames(&self, value: f64) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_frames(checked_usize(value)?);
        Ok(())
    }
    #[napi(getter, js_name = "columns")]
    pub fn binding_columns(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .columns() as f64)
    }
    #[napi(setter, js_name = "columns")]
    pub fn binding_set_columns(&self, value: f64) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_columns(checked_usize(value)?);
        Ok(())
    }
    #[napi(js_name = "values")]
    pub fn binding_values(&self) -> napi::Result<napi::bindgen_prelude::Float32Array> {
        Ok(<napi::bindgen_prelude::Float32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .values(),
        ))
    }
    #[napi(js_name = "set_values")]
    pub fn binding_set_values(
        &self,
        values: napi::bindgen_prelude::Float32Array,
    ) -> napi::Result<()> {
        let storage_values = values.to_vec();
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_values(&storage_values);
        Ok(())
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(Features {
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
