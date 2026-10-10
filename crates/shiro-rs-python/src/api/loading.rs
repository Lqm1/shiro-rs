use super::{Dataset, Model, SegmentationDocument, error};
use pyo3::PyErr as BindingError;
use shiro_rs::hsmm::{Dataset as NativeDataset, Observation};
use std::{collections::BTreeMap, io};
/// Feature-file snapshots addressed by the exact document filename.
#[derive(Clone, Default)]
pub struct FeatureFiles {
    inner: BTreeMap<String, Vec<u8>>,
}
impl FeatureFiles {
    pub fn new() -> Self {
        Self::default()
    }
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
impl Dataset {
    pub fn load(
        document: &SegmentationDocument,
        model: &Model,
        files: &FeatureFiles,
        maximum_frames: usize,
    ) -> Result<Self, BindingError> {
        crate::dataset::load_resolved(&document.inner, &model.inner, |name, dimensions| {
            files.resolve(name, dimensions, maximum_frames)
        })
        .map(|inner| Self { inner })
        .map_err(error)
    }
}
#[derive(Clone, Default)]
pub struct Datasets {
    pub(super) inner: Vec<NativeDataset>,
}
impl Datasets {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<Dataset> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| Dataset { inner })
    }
    pub fn push(&mut self, value: &Dataset) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &Dataset) -> Result<(), BindingError> {
        *self
            .inner
            .get_mut(index)
            .ok_or_else(|| error("collection index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
impl Datasets {
    pub(super) fn native(&self) -> &[NativeDataset] {
        &self.inner
    }
}
impl Datasets {
    pub fn load_training_files(
        document: &SegmentationDocument,
        model: &Model,
        files: &FeatureFiles,
        maximum_frames: usize,
        isolated: bool,
    ) -> Result<Self, BindingError> {
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
