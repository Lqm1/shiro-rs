use super::*;
#[napi]
pub struct IterationReport {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::IterationReport>>,
}
#[napi]
impl IterationReport {
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
impl IterationReport {
    #[napi(constructor)]
    pub fn binding_new(
        iteration: f64,
        temperature: f64,
        mean_log_likelihood: f64,
        rows: &LikelihoodRows,
    ) -> napi::Result<Self> {
        Ok(IterationReport {
            inner: std::cell::RefCell::new(Some(crate::api::IterationReport::new(
                checked_usize(iteration)?,
                temperature as f32,
                mean_log_likelihood as f32,
                rows.inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(getter, js_name = "iteration")]
    pub fn binding_iteration(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .iteration() as f64)
    }
    #[napi(setter, js_name = "iteration")]
    pub fn binding_set_iteration(&self, value: f64) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_iteration(checked_usize(value)?);
        Ok(())
    }
    #[napi(getter, js_name = "temperature")]
    pub fn binding_temperature(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .temperature() as f64)
    }
    #[napi(setter, js_name = "temperature")]
    pub fn binding_set_temperature(&self, value: f64) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_temperature(value as f32);
        Ok(())
    }
    #[napi(getter, js_name = "mean_log_likelihood")]
    pub fn binding_mean_log_likelihood(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .mean_log_likelihood() as f64)
    }
    #[napi(setter, js_name = "mean_log_likelihood")]
    pub fn binding_set_mean_log_likelihood(&self, value: f64) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_mean_log_likelihood(value as f32);
        Ok(())
    }
    #[napi(js_name = "file_likelihoods")]
    pub fn binding_file_likelihoods(&self) -> napi::Result<LikelihoodRows> {
        Ok(LikelihoodRows {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .file_likelihoods(),
            )),
        })
    }
    #[napi(js_name = "set_file_likelihoods")]
    pub fn binding_set_file_likelihoods(&self, rows: &LikelihoodRows) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_file_likelihoods(
            rows.inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(IterationReport {
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
