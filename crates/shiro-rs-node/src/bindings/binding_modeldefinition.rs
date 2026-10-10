use super::*;
#[napi]
pub struct ModelDefinition {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::ModelDefinition>>,
}
#[napi]
impl ModelDefinition {
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
impl ModelDefinition {
    #[napi(constructor)]
    pub fn binding_new(json: String) -> napi::Result<Self> {
        (crate::api::ModelDefinition::new(&json)).map(|value| ModelDefinition {
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
        Ok(ModelDefinition {
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
impl ModelDefinition {
    #[napi(js_name = "build")]
    pub fn binding_build(&self) -> napi::Result<Model> {
        ((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .build())
        .map(|value| Model {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
}
