use super::{PhoneMap, States, error};
use crate::labels::{self, Label as NativeLabel};
use pyo3::PyErr as BindingError;
#[derive(Clone)]
pub struct Label {
    pub start: f64,
    pub end: f64,
    name: String,
}
impl Label {
    pub fn new(start: f64, end: f64, name: String) -> Self {
        Self { start, end, name }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
impl Label {
    pub(super) fn native(&self) -> NativeLabel {
        NativeLabel {
            start: self.start,
            end: self.end,
            name: self.name.clone(),
        }
    }
}
#[derive(Clone, Default)]
pub struct Labels {
    pub(super) inner: Vec<NativeLabel>,
}
impl Labels {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn parse(text: &str) -> Result<Self, BindingError> {
        labels::parse(text)
            .map(|inner| Self { inner })
            .map_err(error)
    }
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
    pub fn replace(&mut self, index: usize, label: &Label) -> Result<(), BindingError> {
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
    pub fn write(&self) -> Result<Vec<u8>, BindingError> {
        let mut bytes = Vec::new();
        labels::write(&self.inner, &mut bytes).map_err(error)?;
        Ok(bytes)
    }
    pub fn to_states(&self, map: &PhoneMap, hop: f64) -> Result<States, BindingError> {
        labels::to_states(&self.inner, &map.inner, hop)
            .map(|inner| States { inner })
            .map_err(error)
    }
    pub fn from_states(
        states: &States,
        hop: f64,
        include_states: bool,
    ) -> Result<Self, BindingError> {
        labels::from_states(&states.inner, hop, include_states)
            .map(|inner| Self { inner })
            .map_err(error)
    }
}
pub fn label_output_path(filename: &str, suffix: &str) -> String {
    labels::output_path(filename, suffix)
        .to_string_lossy()
        .into_owned()
}
