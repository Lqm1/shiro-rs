use super::{ModelDefinition, PhoneMap, States, error};
use crate::{phonemap, segmentation};
use wasm_bindgen::prelude::*;

/// Complete editable native phone expansion options.
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct PhoneMapOptions {
    pub states_per_phone: usize,
    pub streams: usize,
    pub topology: Option<String>,
    pub weak_skips: bool,
}

impl Default for PhoneMapOptions {
    fn default() -> Self {
        let options = phonemap::Options::default();
        Self {
            states_per_phone: options.states_per_phone,
            streams: options.streams,
            topology: options.topology,
            weak_skips: options.weak_skips,
        }
    }
}

impl PhoneMapOptions {
    fn native(&self) -> phonemap::Options {
        phonemap::Options {
            states_per_phone: self.states_per_phone,
            streams: self.streams,
            topology: self.topology.clone(),
            weak_skips: self.weak_skips,
        }
    }
}

#[wasm_bindgen]
impl PhoneMapOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cloned(&self) -> Self {
        self.clone()
    }
}

#[wasm_bindgen]
impl PhoneMap {
    pub fn create(text: &str, options: &PhoneMapOptions) -> Result<Self, JsValue> {
        phonemap::create(text, &options.native())
            .map(|inner| Self { inner })
            .map_err(error)
    }

    pub fn to_definition(&self, dimensions: usize, hop: f64) -> Result<ModelDefinition, JsValue> {
        phonemap::to_definition(&self.inner, dimensions, hop)
            .map(|inner| ModelDefinition { inner })
            .map_err(error)
    }

    pub fn initial(&self, phones: Vec<String>, frames: usize) -> Result<States, JsValue> {
        segmentation::initial(&phones, &self.inner, frames)
            .map(|inner| States { inner })
            .map_err(error)
    }
}
