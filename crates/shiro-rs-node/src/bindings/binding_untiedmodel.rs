use super::*;
#[napi]
pub struct UntiedModel {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::UntiedModel>>,
}
#[napi]
impl UntiedModel {
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
impl UntiedModel {
    #[napi(constructor)]
    pub fn binding_new(
        model: &Model,
        segmentation: &SegmentationDocument,
        assignments: &Assignments,
    ) -> napi::Result<Self> {
        Ok(UntiedModel {
            inner: std::cell::RefCell::new(Some(crate::api::UntiedModel::new(
                model
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                segmentation
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                assignments
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
    pub fn binding_set_model(&self, value: &Model) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_model(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "segmentation")]
    pub fn binding_segmentation(&self) -> napi::Result<SegmentationDocument> {
        Ok(SegmentationDocument {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .segmentation(),
            )),
        })
    }
    #[napi(js_name = "set_segmentation")]
    pub fn binding_set_segmentation(&self, value: &SegmentationDocument) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_segmentation(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "assignments")]
    pub fn binding_assignments(&self) -> napi::Result<Assignments> {
        Ok(Assignments {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .assignments(),
            )),
        })
    }
    #[napi(js_name = "set_assignments")]
    pub fn binding_set_assignments(&self, value: &Assignments) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_assignments(
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
        Ok(UntiedModel {
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
    #[napi(js_name = "write_summary")]
    pub fn binding_write_summary(&self) -> napi::Result<napi::bindgen_prelude::Uint8Array> {
        ((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .write_summary())
        .map(<napi::bindgen_prelude::Uint8Array>::from)
    }
}
