use super::*;
#[napi]
pub struct Gaussian {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Gaussian>>,
}
#[napi]
impl Gaussian {
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
impl Gaussian {
    #[napi(constructor)]
    pub fn binding_new(components: f64, dimensions: f64) -> napi::Result<Self> {
        (crate::api::Gaussian::new(checked_usize(components)?, checked_usize(dimensions)?)).map(
            |value| Gaussian {
                inner: std::cell::RefCell::new(Some(value)),
            },
        )
    }
    #[napi(getter, js_name = "dimensions")]
    pub fn binding_dimensions(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .dimensions() as f64)
    }
    #[napi(getter, js_name = "components")]
    pub fn binding_components(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .components() as f64)
    }
    #[napi(js_name = "weights")]
    pub fn binding_weights(&self) -> napi::Result<napi::bindgen_prelude::Float32Array> {
        Ok(<napi::bindgen_prelude::Float32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .weights(),
        ))
    }
    #[napi(js_name = "means")]
    pub fn binding_means(&self) -> napi::Result<napi::bindgen_prelude::Float32Array> {
        Ok(<napi::bindgen_prelude::Float32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .means(),
        ))
    }
    #[napi(js_name = "variances")]
    pub fn binding_variances(&self) -> napi::Result<napi::bindgen_prelude::Float32Array> {
        Ok(<napi::bindgen_prelude::Float32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .variances(),
        ))
    }
    #[napi(js_name = "variance_floors")]
    pub fn binding_variance_floors(&self) -> napi::Result<napi::bindgen_prelude::Float32Array> {
        Ok(<napi::bindgen_prelude::Float32Array>::from(
            (self
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?)
            .variance_floors(),
        ))
    }
    #[napi(js_name = "replace")]
    pub fn binding_replace(
        &self,
        dimensions: f64,
        weights: napi::bindgen_prelude::Float32Array,
        means: napi::bindgen_prelude::Float32Array,
        variances: napi::bindgen_prelude::Float32Array,
        variance_floors: napi::bindgen_prelude::Float32Array,
    ) -> napi::Result<()> {
        let storage_weights = weights.to_vec();
        let storage_means = means.to_vec();
        let storage_variances = variances.to_vec();
        let storage_variance_floors = variance_floors.to_vec();
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .replace(
            checked_usize(dimensions)?,
            &storage_weights,
            &storage_means,
            &storage_variances,
            &storage_variance_floors,
        )
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(Gaussian {
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
