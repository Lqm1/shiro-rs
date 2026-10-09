use super::{PhoneMap, States, error};
use crate::labels::{self, Label as NativeLabel};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Label {
    pub start: f64,
    pub end: f64,
    name: String,
}

#[wasm_bindgen]
impl Label {
    #[wasm_bindgen(constructor)]
    pub fn new(start: f64, end: f64, name: String) -> Self {
        Self { start, end, name }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn cloned(&self) -> Self {
        self.clone()
    }
}

impl Label {
    fn native(&self) -> NativeLabel {
        NativeLabel {
            start: self.start,
            end: self.end,
            name: self.name.clone(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Labels {
    inner: Vec<NativeLabel>,
}

#[wasm_bindgen]
impl Labels {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(text: &str) -> Result<Self, JsValue> {
        labels::parse(text)
            .map(|inner| Self { inner })
            .map_err(error)
    }

    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.inner.len()
    }

    pub fn get(&self, index: usize) -> Option<Label> {
        self.inner
            .get(index)
            .map(|label| Label::new(label.start, label.end, label.name.clone()))
    }

    pub fn push(&mut self, label: &Label) {
        self.inner.push(label.native());
    }

    pub fn replace(&mut self, index: usize, label: &Label) -> Result<(), JsValue> {
        let target = self
            .inner
            .get_mut(index)
            .ok_or_else(|| error("label index out of range"))?;
        *target = label.native();
        Ok(())
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn cloned(&self) -> Self {
        self.clone()
    }

    pub fn write(&self) -> Result<Vec<u8>, JsValue> {
        let mut bytes = Vec::new();
        labels::write(&self.inner, &mut bytes).map_err(error)?;
        Ok(bytes)
    }

    pub fn to_states(&self, map: &PhoneMap, hop: f64) -> Result<States, JsValue> {
        labels::to_states(&self.inner, &map.inner, hop)
            .map(|inner| States { inner })
            .map_err(error)
    }

    pub fn from_states(states: &States, hop: f64, include_states: bool) -> Result<Self, JsValue> {
        labels::from_states(&states.inner, hop, include_states)
            .map(|inner| Self { inner })
            .map_err(error)
    }
}

#[wasm_bindgen]
pub fn label_output_path(filename: &str, suffix: &str) -> String {
    labels::output_path(filename, suffix)
        .to_string_lossy()
        .into_owned()
}
