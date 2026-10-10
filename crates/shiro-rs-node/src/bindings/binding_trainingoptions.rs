use super::*;
#[napi]
pub struct TrainingOptions {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::TrainingOptions>>,
}
#[napi]
impl TrainingOptions {
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
impl TrainingOptions {
    #[napi(getter, js_name = "iterations")]
    pub fn binding_get_iterations(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .iterations as f64)
    }
    #[napi(setter, js_name = "iterations")]
    pub fn binding_set_iterations(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .iterations = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "duration_mode")]
    pub fn binding_get_duration_mode(&self) -> napi::Result<u32> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .duration_mode)
    }
    #[napi(setter, js_name = "duration_mode")]
    pub fn binding_set_duration_mode(&self, value: u32) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .duration_mode = value;
        Ok(())
    }
    #[napi(getter, js_name = "hsmm_temperature")]
    pub fn binding_get_hsmm_temperature(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .hsmm_temperature as f64)
    }
    #[napi(setter, js_name = "hsmm_temperature")]
    pub fn binding_set_hsmm_temperature(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .hsmm_temperature = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "duration_weight")]
    pub fn binding_get_duration_weight(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .duration_weight as f64)
    }
    #[napi(setter, js_name = "duration_weight")]
    pub fn binding_set_duration_weight(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .duration_weight = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "state_radius")]
    pub fn binding_get_state_radius(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .state_radius as f64)
    }
    #[napi(setter, js_name = "state_radius")]
    pub fn binding_set_state_radius(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .state_radius = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "duration_extra")]
    pub fn binding_get_duration_extra(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .duration_extra as f64)
    }
    #[napi(setter, js_name = "duration_extra")]
    pub fn binding_set_duration_extra(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .duration_extra = checked_usize(value)?;
        Ok(())
    }
    #[napi(getter, js_name = "duration_extra_factor")]
    pub fn binding_get_duration_extra_factor(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .duration_extra_factor as f64)
    }
    #[napi(setter, js_name = "duration_extra_factor")]
    pub fn binding_set_duration_extra_factor(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .duration_extra_factor = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "geometric_temperature")]
    pub fn binding_get_geometric_temperature(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .geometric_temperature as f64)
    }
    #[napi(setter, js_name = "geometric_temperature")]
    pub fn binding_set_geometric_temperature(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .geometric_temperature = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "pruning_slope")]
    pub fn binding_get_pruning_slope(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .pruning_slope as f64)
    }
    #[napi(setter, js_name = "pruning_slope")]
    pub fn binding_set_pruning_slope(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .pruning_slope = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "termination_threshold")]
    pub fn binding_get_termination_threshold(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .termination_threshold as f64)
    }
    #[napi(setter, js_name = "termination_threshold")]
    pub fn binding_set_termination_threshold(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .termination_threshold = value as f32;
        Ok(())
    }
    #[napi(getter, js_name = "deterministic_annealing")]
    pub fn binding_get_deterministic_annealing(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .deterministic_annealing)
    }
    #[napi(setter, js_name = "deterministic_annealing")]
    pub fn binding_set_deterministic_annealing(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .deterministic_annealing = value;
        Ok(())
    }
    #[napi(getter, js_name = "mean_frame_likelihood")]
    pub fn binding_get_mean_frame_likelihood(&self) -> napi::Result<bool> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .mean_frame_likelihood)
    }
    #[napi(setter, js_name = "mean_frame_likelihood")]
    pub fn binding_set_mean_frame_likelihood(&self, value: bool) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .mean_frame_likelihood = value;
        Ok(())
    }
    #[napi(getter, js_name = "workers")]
    pub fn binding_get_workers(&self) -> napi::Result<f64> {
        Ok(self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?
            .workers as f64)
    }
    #[napi(setter, js_name = "workers")]
    pub fn binding_set_workers(&self, value: f64) -> napi::Result<()> {
        self.inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?
            .workers = checked_usize(value)?;
        Ok(())
    }
}
#[napi]
impl TrainingOptions {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(TrainingOptions {
            inner: std::cell::RefCell::new(Some(crate::api::TrainingOptions::new())),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(TrainingOptions {
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
