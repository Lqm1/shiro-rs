use super::*;
#[napi]
pub struct SegmentedUtterances {
    pub(crate) inner: std::cell::RefCell<Option<crate::api::SegmentedUtterances>>,
}
#[napi]
impl SegmentedUtterances {
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
impl SegmentedUtterances {
    #[napi(constructor)]
    pub fn binding_new(
        model: &Model,
        phonemap: &PhoneMap,
        definition: &ModelDefinition,
        initial_segmentation: &SegmentationDocument,
        alignment: &SegmentationDocument,
    ) -> napi::Result<Self> {
        Ok(SegmentedUtterances {
            inner: std::cell::RefCell::new(Some(crate::api::SegmentedUtterances::new(
                model
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                phonemap
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                definition
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                initial_segmentation
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
                alignment
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?,
            ))),
        })
    }
    #[napi(js_name = "split_features")]
    pub fn binding_split_features(
        features: &Features,
        filename: String,
        options: &UtteranceOptions,
        source: &UtteranceModelSource,
    ) -> napi::Result<Self> {
        (crate::api::SegmentedUtterances::split_features(
            features
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            &filename,
            options
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
            source
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        ))
        .map(|value| SegmentedUtterances {
            inner: std::cell::RefCell::new(Some(value)),
        })
    }
    #[napi(js_name = "cloned")]
    pub fn binding_cloned(&self) -> napi::Result<Self> {
        Ok(SegmentedUtterances {
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
    #[napi(js_name = "phones_json")]
    pub fn binding_phones_json(&self) -> napi::Result<String> {
        (self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .phones_json()
    }
    #[napi(js_name = "set_phones_json")]
    pub fn binding_set_phones_json(&self, json: String) -> napi::Result<()> {
        (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_phones_json(&json)
    }
    #[napi(js_name = "uninitialized_model")]
    pub fn binding_uninitialized_model(&self) -> napi::Result<Option<Model>> {
        Ok(((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .uninitialized_model())
        .map(|value| Model {
            inner: std::cell::RefCell::new(Some(value)),
        }))
    }
    #[napi(js_name = "set_uninitialized_model")]
    pub fn binding_set_uninitialized_model(&self, value: &Model) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_uninitialized_model(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "clear_uninitialized_model")]
    pub fn binding_clear_uninitialized_model(&self) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .clear_uninitialized_model();
        Ok(())
    }
    #[napi(js_name = "initialized_model")]
    pub fn binding_initialized_model(&self) -> napi::Result<Option<Model>> {
        Ok(((self
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?)
        .initialized_model())
        .map(|value| Model {
            inner: std::cell::RefCell::new(Some(value)),
        }))
    }
    #[napi(js_name = "set_initialized_model")]
    pub fn binding_set_initialized_model(&self, value: &Model) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_initialized_model(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
    #[napi(js_name = "clear_initialized_model")]
    pub fn binding_clear_initialized_model(&self) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .clear_initialized_model();
        Ok(())
    }
}
#[napi]
impl SegmentedUtterances {
    #[napi(js_name = "phonemap")]
    pub fn binding_phonemap(&self) -> napi::Result<PhoneMap> {
        Ok(PhoneMap {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .phonemap(),
            )),
        })
    }
    #[napi(js_name = "set_phonemap")]
    pub fn binding_set_phonemap(&self, value: &PhoneMap) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_phonemap(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
}
#[napi]
impl SegmentedUtterances {
    #[napi(js_name = "definition")]
    pub fn binding_definition(&self) -> napi::Result<ModelDefinition> {
        Ok(ModelDefinition {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .definition(),
            )),
        })
    }
    #[napi(js_name = "set_definition")]
    pub fn binding_set_definition(&self, value: &ModelDefinition) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_definition(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
}
#[napi]
impl SegmentedUtterances {
    #[napi(js_name = "initial_segmentation")]
    pub fn binding_initial_segmentation(&self) -> napi::Result<SegmentationDocument> {
        Ok(SegmentationDocument {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .initial_segmentation(),
            )),
        })
    }
    #[napi(js_name = "set_initial_segmentation")]
    pub fn binding_set_initial_segmentation(
        &self,
        value: &SegmentationDocument,
    ) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_initial_segmentation(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
}
#[napi]
impl SegmentedUtterances {
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
}
#[napi]
impl SegmentedUtterances {
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
    #[napi(js_name = "set_iterations")]
    pub fn binding_set_iterations(&self, value: &IterationReports) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_iterations(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
}
#[napi]
impl SegmentedUtterances {
    #[napi(js_name = "alignment")]
    pub fn binding_alignment(&self) -> napi::Result<SegmentationDocument> {
        Ok(SegmentationDocument {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .alignment(),
            )),
        })
    }
    #[napi(js_name = "set_alignment")]
    pub fn binding_set_alignment(&self, value: &SegmentationDocument) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_alignment(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
}
#[napi]
impl SegmentedUtterances {
    #[napi(js_name = "labels")]
    pub fn binding_labels(&self) -> napi::Result<Labels> {
        Ok(Labels {
            inner: std::cell::RefCell::new(Some(
                (self
                    .inner
                    .try_borrow()
                    .map_err(borrowed)?
                    .as_ref()
                    .ok_or_else(consumed)?)
                .labels(),
            )),
        })
    }
    #[napi(js_name = "set_labels")]
    pub fn binding_set_labels(&self, value: &Labels) -> napi::Result<()> {
        let _: () = (self
            .inner
            .try_borrow_mut()
            .map_err(borrowed)?
            .as_mut()
            .ok_or_else(consumed)?)
        .set_labels(
            value
                .inner
                .try_borrow()
                .map_err(borrowed)?
                .as_ref()
                .ok_or_else(consumed)?,
        );
        Ok(())
    }
}
