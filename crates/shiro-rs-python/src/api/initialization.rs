use super::{Dataset, Model, error};
use crate::initialization;
use pyo3::PyErr as BindingError;
#[derive(Clone, Copy)]
pub struct InitializationOptions {
    pub flat_start: bool,
    pub globally_tied: bool,
    pub variance_floor_ratio: f32,
}
impl Default for InitializationOptions {
    fn default() -> Self {
        let options = initialization::Options::default();
        Self {
            flat_start: options.flat_start,
            globally_tied: options.globally_tied,
            variance_floor_ratio: options.variance_floor_ratio,
        }
    }
}
impl InitializationOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}
impl Model {
    pub fn initialize(
        &self,
        dataset: &Dataset,
        options: &InitializationOptions,
    ) -> Result<Model, BindingError> {
        initialization::initialize(
            &self.inner,
            &dataset.inner,
            initialization::Options {
                flat_start: options.flat_start,
                globally_tied: options.globally_tied,
                variance_floor_ratio: options.variance_floor_ratio,
            },
        )
        .map(|inner| Model { inner })
        .map_err(error)
    }
}
