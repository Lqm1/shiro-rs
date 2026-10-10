use super::error;
use crate::index::{self, Entry};
use napi::Error as BindingError;
use std::{io::Cursor, path::Path};
#[derive(Clone)]
pub struct IndexEntry {
    inner: Entry,
}
impl IndexEntry {
    pub fn new(stem: &str, phonemes_json: &str) -> Result<Self, BindingError> {
        Ok(Self {
            inner: Entry {
                stem: stem.into(),
                phonemes: serde_json::from_str(phonemes_json).map_err(error)?,
            },
        })
    }
    pub fn stem(&self) -> Result<String, BindingError> {
        self.inner
            .stem
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| error("index stem is not UTF-8"))
    }
    pub fn set_stem(&mut self, stem: &str) {
        self.inner.stem = stem.into();
    }
    pub fn phonemes_json(&self) -> Result<String, BindingError> {
        serde_json::to_string(&self.inner.phonemes).map_err(error)
    }
    pub fn set_phonemes_json(&mut self, json: &str) -> Result<(), BindingError> {
        let phonemes = serde_json::from_str(json).map_err(error)?;
        self.inner.phonemes = phonemes;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone, Default)]
pub struct IndexEntries {
    pub(super) inner: Vec<Entry>,
}
impl IndexEntries {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<IndexEntry> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| IndexEntry { inner })
    }
    pub fn push(&mut self, value: &IndexEntry) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &IndexEntry) -> Result<(), BindingError> {
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
impl IndexEntries {
    pub fn read(
        bytes: &[u8],
        directory: &str,
        left_json: &str,
        right_json: &str,
    ) -> Result<Self, BindingError> {
        let left: Vec<String> = serde_json::from_str(left_json).map_err(error)?;
        let right: Vec<String> = serde_json::from_str(right_json).map_err(error)?;
        index::read(Cursor::new(bytes), Path::new(directory), &left, &right)
            .map(|inner| Self { inner })
            .map_err(error)
    }
}
pub fn index_append_suffix(path: &str, suffix: &str) -> Result<String, BindingError> {
    index::append_suffix(Path::new(path), suffix)
        .into_os_string()
        .into_string()
        .map_err(|_| error("index path is not UTF-8"))
}
