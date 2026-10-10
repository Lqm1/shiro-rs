use super::*;
#[napi]
pub struct TrainingResult {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::TrainingResult>>,
}
#[napi]
impl TrainingResult {
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
impl TrainingResult {
    #[napi(constructor)]
    pub fn binding_new(model: &Model, iterations: &IterationReports) -> napi::Result<Self> {
        Ok(TrainingResult {
            inner: std::cell::RefCell::new(Some(crate::api::TrainingResult::new(
                model
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                iterations
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(js_name = "model")]
    pub fn binding_model(&self) -> napi::Result<Model> {
        Ok(Model {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .model(),
            )),
        })
    }
    #[napi(js_name = "set_model")]
    pub fn binding_set_model(&self, model: &Model) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_model(
            model
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "iterations")]
    pub fn binding_iterations(&self) -> napi::Result<IterationReports> {
        Ok(IterationReports {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .iterations(),
            )),
        })
    }
    #[napi(js_name = "write_likelihood_csv")]
    pub fn binding_write_likelihood_csv(&self) -> napi::Result<napi::bindgen_prelude::Uint8Array> {
        ((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .write_likelihood_csv())
        .map(<napi::bindgen_prelude::Uint8Array>::from)
    }
    #[napi(js_name = "set_iterations")]
    pub fn binding_set_iterations(&self, iterations: &IterationReports) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_iterations(
            iterations
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
        Ok(TrainingResult {
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
