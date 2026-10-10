use super::*;
#[napi]
pub struct Dataset {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::Dataset>>,
}
#[napi]
impl Dataset {
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
impl Dataset {
    #[napi(constructor)]
    pub fn binding_new() -> napi::Result<Self> {
        Ok(Dataset {
            inner: std::cell::RefCell::new(Some(crate::api::Dataset::new())),
        })
    }
    #[napi(js_name = "observations")]
    pub fn binding_observations(&self) -> napi::Result<Observations> {
        Ok(Observations {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .observations(),
            )),
        })
    }
    #[napi(js_name = "segmentations")]
    pub fn binding_segmentations(&self) -> napi::Result<Segmentations> {
        Ok(Segmentations {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .segmentations(),
            )),
        })
    }
    #[napi(js_name = "set_observations")]
    pub fn binding_set_observations(&self, values: &Observations) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_observations(
            values
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "set_segmentations")]
    pub fn binding_set_segmentations(&self, values: &Segmentations) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_segmentations(
            values
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "replace")]
    pub fn binding_replace(
        &self,
        observations: &Observations,
        segmentations: &Segmentations,
    ) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .replace(
            observations
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            segmentations
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
        Ok(Dataset {
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
impl Dataset {
    #[napi(js_name = "load_files")]
    pub fn binding_load_files(
        document: &SegmentationDocument,
        model: &Model,
        maximum_frames: f64,
    ) -> napi::Result<Self> {
        (crate::api::Dataset::load_files(
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
        ))
        .map(|value| Dataset {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
}
#[napi]
impl Dataset {
    #[napi(js_name = "load")]
    pub fn binding_load(
        document: &SegmentationDocument,
        model: &Model,
        files: &FeatureFiles,
        maximum_frames: f64,
    ) -> napi::Result<Self> {
        (crate::api::Dataset::load(
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
        ))
        .map(|value| Dataset {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
}
