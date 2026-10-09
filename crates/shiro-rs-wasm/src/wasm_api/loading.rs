use super::models::collection;
use super::{Dataset, Model, SegmentationDocument, error};
use shiro_rs::hsmm::{Dataset as NativeDataset, Observation};
use std::{collections::BTreeMap, io};
use wasm_bindgen::prelude::*;

/// Feature-file snapshots addressed by the exact document filename.
#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct FeatureFiles {
    inner: BTreeMap<String, Vec<u8>>,
}

#[wasm_bindgen]
impl FeatureFiles {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn names(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }
    pub fn get(&self, filename: &str) -> Option<Vec<u8>> {
        self.inner.get(filename).cloned()
    }
    pub fn set(&mut self, filename: &str, bytes: &[u8]) {
        self.inner.insert(filename.into(), bytes.to_vec());
    }
    pub fn remove(&mut self, filename: &str) -> bool {
        self.inner.remove(filename).is_some()
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}

impl FeatureFiles {
    pub(super) fn resolve(
        &self,
        filename: &str,
        dimensions: &[usize],
        maximum_frames: usize,
    ) -> io::Result<Observation> {
        let bytes = self
            .inner
            .get(filename)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "feature file is missing"))?;
        crate::dataset::read_observation(bytes.as_slice(), dimensions, maximum_frames)
    }
}

#[wasm_bindgen]
impl Dataset {
    pub fn load(
        document: &SegmentationDocument,
        model: &Model,
        files: &FeatureFiles,
        maximum_frames: usize,
    ) -> Result<Self, JsValue> {
        crate::dataset::load_resolved(&document.inner, &model.inner, |name, dimensions| {
            files.resolve(name, dimensions, maximum_frames)
        })
        .map(|inner| Self { inner })
        .map_err(error)
    }
}

collection!(Datasets, Dataset, NativeDataset);

impl Datasets {
    pub(super) fn native(&self) -> &[NativeDataset] {
        &self.inner
    }
}

#[wasm_bindgen]
impl Datasets {
    pub fn load_training_files(
        document: &SegmentationDocument,
        model: &Model,
        files: &FeatureFiles,
        maximum_frames: usize,
        isolated: bool,
    ) -> Result<Self, JsValue> {
        crate::dataset::load_training_resolved(
            &document.inner,
            &model.inner,
            isolated,
            |name, dimensions| files.resolve(name, dimensions, maximum_frames),
        )
        .map(|inner| Self { inner })
        .map_err(error)
    }
}
