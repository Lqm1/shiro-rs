use super::{error, models::collection};
use crate::index::{self, Entry};
use std::{io::Cursor, path::Path};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct IndexEntry {
    inner: Entry,
}
#[wasm_bindgen]
impl IndexEntry {
    #[wasm_bindgen(constructor)]
    pub fn new(stem: &str, phonemes_json: &str) -> Result<Self, JsValue> {
        Ok(Self {
            inner: Entry {
                stem: stem.into(),
                phonemes: serde_json::from_str(phonemes_json).map_err(error)?,
            },
        })
    }
    #[wasm_bindgen(getter)]
    pub fn stem(&self) -> Result<String, JsValue> {
        self.inner
            .stem
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| error("index stem is not UTF-8"))
    }
    #[wasm_bindgen(setter)]
    pub fn set_stem(&mut self, stem: &str) {
        self.inner.stem = stem.into();
    }
    pub fn phonemes_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner.phonemes).map_err(error)
    }
    pub fn set_phonemes_json(&mut self, json: &str) -> Result<(), JsValue> {
        let phonemes = serde_json::from_str(json).map_err(error)?;
        self.inner.phonemes = phonemes;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
collection!(IndexEntries, IndexEntry, Entry);
#[wasm_bindgen]
impl IndexEntries {
    pub fn read(
        bytes: &[u8],
        directory: &str,
        left_json: &str,
        right_json: &str,
    ) -> Result<Self, JsValue> {
        let left: Vec<String> = serde_json::from_str(left_json).map_err(error)?;
        let right: Vec<String> = serde_json::from_str(right_json).map_err(error)?;
        index::read(Cursor::new(bytes), Path::new(directory), &left, &right)
            .map(|inner| Self { inner })
            .map_err(error)
    }
}
#[wasm_bindgen]
pub fn index_append_suffix(path: &str, suffix: &str) -> Result<String, JsValue> {
    index::append_suffix(Path::new(path), suffix)
        .into_os_string()
        .into_string()
        .map_err(|_| error("index path is not UTF-8"))
}
