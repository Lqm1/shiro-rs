use super::*;
#[pyclass(module = "shiro_rs")]
pub struct SegmentedUtterances {
    pub(crate) inner: Option<crate::api::SegmentedUtterances>,
}
#[pymethods]
impl SegmentedUtterances {
    fn close(&mut self) {
        self.inner = None;
    }
    fn free(&mut self) {
        self.inner = None;
    }
    #[getter]
    fn is_closed(&self) -> bool {
        self.inner.is_none()
    }
}
#[pymethods]
impl SegmentedUtterances {
    #[new]
    #[pyo3(signature = (model, phonemap, definition, initial_segmentation, alignment))]
    pub fn binding_new(
        model: pyo3::PyRef<'_, Model>,
        phonemap: pyo3::PyRef<'_, PhoneMap>,
        definition: pyo3::PyRef<'_, ModelDefinition>,
        initial_segmentation: pyo3::PyRef<'_, SegmentationDocument>,
        alignment: pyo3::PyRef<'_, SegmentationDocument>,
    ) -> pyo3::PyResult<Self> {
        Ok(SegmentedUtterances {
            inner: Some(crate::api::SegmentedUtterances::new(
                model.inner.as_ref().ok_or_else(consumed)?,
                phonemap.inner.as_ref().ok_or_else(consumed)?,
                definition.inner.as_ref().ok_or_else(consumed)?,
                initial_segmentation.inner.as_ref().ok_or_else(consumed)?,
                alignment.inner.as_ref().ok_or_else(consumed)?,
            )),
        })
    }
    #[staticmethod]
    #[pyo3(name = "split_features")]
    #[pyo3(signature = (features, filename, options, source))]
    pub fn binding_split_features(
        features: pyo3::PyRef<'_, Features>,
        filename: &str,
        options: pyo3::PyRef<'_, UtteranceOptions>,
        source: pyo3::PyRef<'_, UtteranceModelSource>,
    ) -> pyo3::PyResult<Self> {
        (crate::api::SegmentedUtterances::split_features(
            features.inner.as_ref().ok_or_else(consumed)?,
            filename,
            options.inner.as_ref().ok_or_else(consumed)?,
            source.inner.as_ref().ok_or_else(consumed)?,
        ))
        .map(|value| SegmentedUtterances { inner: Some(value) })
    }
    #[pyo3(name = "cloned")]
    #[pyo3(signature = ())]
    pub fn binding_cloned(&self) -> pyo3::PyResult<Self> {
        Ok(SegmentedUtterances {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).cloned()),
        })
    }
    #[pyo3(name = "phones_json")]
    #[pyo3(signature = ())]
    pub fn binding_phones_json(&self) -> pyo3::PyResult<String> {
        (self.inner.as_ref().ok_or_else(consumed)?).phones_json()
    }
    #[pyo3(name = "set_phones_json")]
    #[pyo3(signature = (json))]
    pub fn binding_set_phones_json(&mut self, json: &str) -> pyo3::PyResult<()> {
        (self.inner.as_mut().ok_or_else(consumed)?).set_phones_json(json)
    }
    #[pyo3(name = "uninitialized_model")]
    #[pyo3(signature = ())]
    pub fn binding_uninitialized_model(&self) -> pyo3::PyResult<Option<Model>> {
        Ok(
            ((self.inner.as_ref().ok_or_else(consumed)?).uninitialized_model())
                .map(|value| Model { inner: Some(value) }),
        )
    }
    #[pyo3(name = "set_uninitialized_model")]
    #[pyo3(signature = (value))]
    pub fn binding_set_uninitialized_model(
        &mut self,
        value: pyo3::PyRef<'_, Model>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_uninitialized_model(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "clear_uninitialized_model")]
    #[pyo3(signature = ())]
    pub fn binding_clear_uninitialized_model(&mut self) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).clear_uninitialized_model();
        Ok(())
    }
    #[pyo3(name = "initialized_model")]
    #[pyo3(signature = ())]
    pub fn binding_initialized_model(&self) -> pyo3::PyResult<Option<Model>> {
        Ok(
            ((self.inner.as_ref().ok_or_else(consumed)?).initialized_model())
                .map(|value| Model { inner: Some(value) }),
        )
    }
    #[pyo3(name = "set_initialized_model")]
    #[pyo3(signature = (value))]
    pub fn binding_set_initialized_model(
        &mut self,
        value: pyo3::PyRef<'_, Model>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_initialized_model(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
    #[pyo3(name = "clear_initialized_model")]
    #[pyo3(signature = ())]
    pub fn binding_clear_initialized_model(&mut self) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?).clear_initialized_model();
        Ok(())
    }
}
#[pymethods]
impl SegmentedUtterances {
    #[pyo3(name = "phonemap")]
    #[pyo3(signature = ())]
    pub fn binding_phonemap(&self) -> pyo3::PyResult<PhoneMap> {
        Ok(PhoneMap {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).phonemap()),
        })
    }
    #[pyo3(name = "set_phonemap")]
    #[pyo3(signature = (value))]
    pub fn binding_set_phonemap(&mut self, value: pyo3::PyRef<'_, PhoneMap>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_phonemap(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
#[pymethods]
impl SegmentedUtterances {
    #[pyo3(name = "definition")]
    #[pyo3(signature = ())]
    pub fn binding_definition(&self) -> pyo3::PyResult<ModelDefinition> {
        Ok(ModelDefinition {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).definition()),
        })
    }
    #[pyo3(name = "set_definition")]
    #[pyo3(signature = (value))]
    pub fn binding_set_definition(
        &mut self,
        value: pyo3::PyRef<'_, ModelDefinition>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_definition(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
#[pymethods]
impl SegmentedUtterances {
    #[pyo3(name = "initial_segmentation")]
    #[pyo3(signature = ())]
    pub fn binding_initial_segmentation(&self) -> pyo3::PyResult<SegmentationDocument> {
        Ok(SegmentationDocument {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).initial_segmentation()),
        })
    }
    #[pyo3(name = "set_initial_segmentation")]
    #[pyo3(signature = (value))]
    pub fn binding_set_initial_segmentation(
        &mut self,
        value: pyo3::PyRef<'_, SegmentationDocument>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_initial_segmentation(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
#[pymethods]
impl SegmentedUtterances {
    #[pyo3(name = "model")]
    #[pyo3(signature = ())]
    pub fn binding_model(&self) -> pyo3::PyResult<Model> {
        Ok(Model {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).model()),
        })
    }
    #[pyo3(name = "set_model")]
    #[pyo3(signature = (value))]
    pub fn binding_set_model(&mut self, value: pyo3::PyRef<'_, Model>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_model(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
#[pymethods]
impl SegmentedUtterances {
    #[pyo3(name = "iterations")]
    #[pyo3(signature = ())]
    pub fn binding_iterations(&self) -> pyo3::PyResult<IterationReports> {
        Ok(IterationReports {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).iterations()),
        })
    }
    #[pyo3(name = "set_iterations")]
    #[pyo3(signature = (value))]
    pub fn binding_set_iterations(
        &mut self,
        value: pyo3::PyRef<'_, IterationReports>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_iterations(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
#[pymethods]
impl SegmentedUtterances {
    #[pyo3(name = "alignment")]
    #[pyo3(signature = ())]
    pub fn binding_alignment(&self) -> pyo3::PyResult<SegmentationDocument> {
        Ok(SegmentationDocument {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).alignment()),
        })
    }
    #[pyo3(name = "set_alignment")]
    #[pyo3(signature = (value))]
    pub fn binding_set_alignment(
        &mut self,
        value: pyo3::PyRef<'_, SegmentationDocument>,
    ) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_alignment(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
#[pymethods]
impl SegmentedUtterances {
    #[pyo3(name = "labels")]
    #[pyo3(signature = ())]
    pub fn binding_labels(&self) -> pyo3::PyResult<Labels> {
        Ok(Labels {
            inner: Some((self.inner.as_ref().ok_or_else(consumed)?).labels()),
        })
    }
    #[pyo3(name = "set_labels")]
    #[pyo3(signature = (value))]
    pub fn binding_set_labels(&mut self, value: pyo3::PyRef<'_, Labels>) -> pyo3::PyResult<()> {
        let _: () = (self.inner.as_mut().ok_or_else(consumed)?)
            .set_labels(value.inner.as_ref().ok_or_else(consumed)?);
        Ok(())
    }
}
