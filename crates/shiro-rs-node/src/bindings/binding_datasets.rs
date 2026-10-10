use super::*;
#[napi]
pub struct Datasets {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Datasets>>,
}
#[napi]
impl Datasets {
    #[napi(js_name = "load_training_paths")]
    pub fn binding_load_training_paths(
        document: &SegmentationDocument,
        model: &Model,
        maximum_frames: f64,
        isolated: bool,
    ) -> napi::Result<Self> {
        (crate::api::Datasets::load_training_paths(
            document
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
            checked_usize(maximum_frames)?,
            isolated,
        ))
        .map(|value| Datasets {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
}
#[napi]
impl Datasets {
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
impl Datasets {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(Datasets {
            inner: std::cell::RefCell::new(Some(crate::api::Datasets::new())),
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
    #[napi(js_name = "get")]
    pub fn binding_get(&self, index: f64) -> napi::Result<Option<Dataset>> {
        Ok(((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .get(checked_usize(index)?))
        .map(|value| Dataset {
            inner: std::cell::RefCell::new(Some(value)),
        }))
    }
    #[napi(js_name = "push")]
    pub fn binding_push(&self, value: &Dataset) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .push(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "replace")]
    pub fn binding_replace(&self, index: f64, value: &Dataset) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .replace(
            checked_usize(index)?,
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        )
    }
    #[napi(js_name = "clear")]
    pub fn binding_clear(&self) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .clear();
        Ok(())
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(Datasets {
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
impl Datasets {
    #[napi(js_name = "load_training_files")]
    pub fn binding_load_training_files(
        document: &SegmentationDocument,
        model: &Model,
        files: &FeatureFiles,
        maximum_frames: f64,
        isolated: bool,
    ) -> napi::Result<Self> {
        (crate::api::Datasets::load_training_files(
            document
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
            files
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            checked_usize(maximum_frames)?,
            isolated,
        ))
        .map(|value| Datasets {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
}
