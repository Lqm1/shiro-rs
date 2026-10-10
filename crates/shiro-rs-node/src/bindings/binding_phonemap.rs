use super::*;
#[napi]
pub struct PhoneMap {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::PhoneMap>>,
}
#[napi]
impl PhoneMap {
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
impl PhoneMap {
    #[napi(constructor)]
    pub fn binding_new(json: String) -> napi::Result<Self> {
        (crate::api::PhoneMap::new(&json)).map(|value| PhoneMap {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "json")]
    pub fn binding_json(&self) -> napi::Result<String> {
        (self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .json()
    }
    #[napi(js_name = "replace")]
    pub fn binding_replace(&self, json: String) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .replace(&json)
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(PhoneMap {
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
impl PhoneMap {
    #[napi(js_name = "create")]
    pub fn binding_create(text: String, options: &PhoneMapOptions) -> napi::Result<Self> {
        (crate::api::PhoneMap::create(
            &text,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        ))
        .map(|value| PhoneMap {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "to_definition")]
    pub fn binding_to_definition(
        &self,
        dimensions: f64,
        hop: f64,
    ) -> napi::Result<ModelDefinition> {
        ((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .to_definition(checked_usize(dimensions)?, hop))
        .map(|value| ModelDefinition {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "initial")]
    pub fn binding_initial(&self, phones: Vec<String>, frames: f64) -> napi::Result<States> {
        ((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .initial(phones.to_vec(), checked_usize(frames)?))
        .map(|value| States {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
}
