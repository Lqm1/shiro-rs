use super::models::collection;
use super::{Model, Observation, States, error};
use crate::dataset::IsolatedGroup as NativeGroup;
use wasm_bindgen::prelude::*;

/// An independent snapshot of one phoneme's input interval and local states.
#[wasm_bindgen]
#[derive(Clone)]
pub struct IsolatedGroup {
    inner: NativeGroup,
}

#[wasm_bindgen]
impl IsolatedGroup {
    #[wasm_bindgen(constructor)]
    pub fn new(
        first_state: usize,
        first_frame: usize,
        observation: &Observation,
        states: &States,
    ) -> Self {
        Self {
            inner: NativeGroup {
                first_state,
                first_frame,
                observation: observation.inner.clone(),
                states: states.inner.clone(),
            },
        }
    }

    #[wasm_bindgen(getter)]
    pub fn first_state(&self) -> usize {
        self.inner.first_state
    }
    #[wasm_bindgen(setter)]
    pub fn set_first_state(&mut self, value: usize) {
        self.inner.first_state = value;
    }
    #[wasm_bindgen(getter)]
    pub fn first_frame(&self) -> usize {
        self.inner.first_frame
    }
    #[wasm_bindgen(setter)]
    pub fn set_first_frame(&mut self, value: usize) {
        self.inner.first_frame = value;
    }
    pub fn observation(&self) -> Observation {
        Observation {
            inner: self.inner.observation.clone(),
        }
    }
    pub fn set_observation(&mut self, value: &Observation) {
        self.inner.observation = value.inner.clone();
    }
    pub fn states(&self) -> States {
        States {
            inner: self.inner.states.clone(),
        }
    }
    pub fn set_states(&mut self, value: &States) {
        self.inner.states = value.inner.clone();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}

collection!(IsolatedGroups, IsolatedGroup, NativeGroup);

#[wasm_bindgen]
impl IsolatedGroups {
    /// Split using the same boundary capping and transition filtering as native
    /// isolated alignment and training. Inputs are retained as independent copies.
    pub fn split(
        model: &Model,
        observation: &Observation,
        states: &States,
    ) -> Result<Self, JsValue> {
        crate::dataset::isolated_groups(&model.inner, &observation.inner, &states.inner)
            .map(|inner| Self { inner })
            .map_err(error)
    }
}
