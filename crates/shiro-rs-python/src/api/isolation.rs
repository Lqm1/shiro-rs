use super::{Model, Observation, States, error};
use crate::dataset::IsolatedGroup as NativeGroup;
use pyo3::PyErr as BindingError;
/// An independent snapshot of one phoneme's input interval and local states.
#[derive(Clone)]
pub struct IsolatedGroup {
    inner: NativeGroup,
}
impl IsolatedGroup {
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
    pub fn first_state(&self) -> usize {
        self.inner.first_state
    }
    pub fn set_first_state(&mut self, value: usize) {
        self.inner.first_state = value;
    }
    pub fn first_frame(&self) -> usize {
        self.inner.first_frame
    }
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
#[derive(Clone, Default)]
pub struct IsolatedGroups {
    pub(super) inner: Vec<NativeGroup>,
}
impl IsolatedGroups {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<IsolatedGroup> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| IsolatedGroup { inner })
    }
    pub fn push(&mut self, value: &IsolatedGroup) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &IsolatedGroup) -> Result<(), BindingError> {
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
impl IsolatedGroups {
    /// Split using the same boundary capping and transition filtering as native
    /// isolated alignment and training. Inputs are retained as independent copies.
    pub fn split(
        model: &Model,
        observation: &Observation,
        states: &States,
    ) -> Result<Self, BindingError> {
        crate::dataset::isolated_groups(&model.inner, &observation.inner, &states.inner)
            .map(|inner| Self { inner })
            .map_err(error)
    }
}
