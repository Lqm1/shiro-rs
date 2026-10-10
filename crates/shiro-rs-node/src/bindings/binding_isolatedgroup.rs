use super::*;
#[napi]
pub struct IsolatedGroup {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::IsolatedGroup>>,
}
#[napi]
impl IsolatedGroup {
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
impl IsolatedGroup {
    #[napi(constructor)]
    pub fn binding_new(
        first_state: f64,
        first_frame: f64,
        observation: &Observation,
        states: &States,
    ) -> napi::Result<Self> {
        Ok(IsolatedGroup {
            inner: std::cell::RefCell::new(Some(crate::api::IsolatedGroup::new(
                checked_usize(first_state)?,
                checked_usize(first_frame)?,
                observation
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                states
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(getter, js_name = "first_state")]
    pub fn binding_first_state(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .first_state() as f64)
    }
    #[napi(setter, js_name = "first_state")]
    pub fn binding_set_first_state(&self, value: f64) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_first_state(checked_usize(value)?);
        Ok(())
    }
    #[napi(getter, js_name = "first_frame")]
    pub fn binding_first_frame(&self) -> napi::Result<f64> {
        Ok((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .first_frame() as f64)
    }
    #[napi(setter, js_name = "first_frame")]
    pub fn binding_set_first_frame(&self, value: f64) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_first_frame(checked_usize(value)?);
        Ok(())
    }
    #[napi(js_name = "observation")]
    pub fn binding_observation(&self) -> napi::Result<Observation> {
        Ok(Observation {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .observation(),
            )),
        })
    }
    #[napi(js_name = "set_observation")]
    pub fn binding_set_observation(&self, value: &Observation) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_observation(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "states")]
    pub fn binding_states(&self) -> napi::Result<States> {
        Ok(States {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .states(),
            )),
        })
    }
    #[napi(js_name = "set_states")]
    pub fn binding_set_states(&self, value: &States) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_states(
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
        Ok(IsolatedGroup {
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
