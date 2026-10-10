use super::{FeatureFiles, Model, Observation, SegmentationDocument, States, error};
use crate::alignment::{self, DurationMode};
use napi::Error as BindingError;
use shiro_rs::hsmm::{GeometricOptions, HsmmOptions};
#[derive(Clone, Copy)]
pub struct AlignmentOptions {
    pub duration_mode: u32,
    pub isolated: bool,
    pub hsmm_temperature: f32,
    pub duration_weight: f32,
    pub state_radius: f32,
    pub duration_extra: usize,
    pub duration_extra_factor: f32,
    pub geometric_temperature: f32,
    pub pruning_slope: f32,
}
impl Default for AlignmentOptions {
    fn default() -> Self {
        let options = alignment::Options::default();
        Self {
            duration_mode: 0,
            isolated: options.isolated,
            hsmm_temperature: options.hsmm.temperature,
            duration_weight: options.hsmm.duration_weight,
            state_radius: options.hsmm.state_radius,
            duration_extra: options.hsmm.duration_extra,
            duration_extra_factor: options.hsmm.duration_extra_factor,
            geometric_temperature: options.geometric.temperature,
            pruning_slope: options.geometric.pruning_slope,
        }
    }
}
impl AlignmentOptions {
    pub(super) fn native(&self) -> Result<alignment::Options, BindingError> {
        Ok(alignment::Options {
            duration_mode: match self.duration_mode {
                0 => DurationMode::Explicit,
                1 => DurationMode::Geometric,
                _ => return Err(error("invalid alignment duration mode")),
            },
            isolated: self.isolated,
            hsmm: HsmmOptions {
                temperature: self.hsmm_temperature,
                duration_weight: self.duration_weight,
                state_radius: self.state_radius,
                duration_extra: self.duration_extra,
                duration_extra_factor: self.duration_extra_factor,
            },
            geometric: GeometricOptions {
                temperature: self.geometric_temperature,
                pruning_slope: self.pruning_slope,
            },
        })
    }
}
impl AlignmentOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}
impl Model {
    pub fn align_states(
        &self,
        observation: &Observation,
        states: &States,
        options: &AlignmentOptions,
    ) -> Result<States, BindingError> {
        alignment::align_states(
            &self.inner,
            &observation.inner,
            &states.inner,
            options.native()?,
        )
        .map(|inner| States { inner })
        .map_err(error)
    }
    pub fn align_document(
        &self,
        document: &SegmentationDocument,
        files: &FeatureFiles,
        maximum_frames: usize,
        options: &AlignmentOptions,
    ) -> Result<SegmentationDocument, BindingError> {
        alignment::align_document_resolved(
            &self.inner,
            &document.inner,
            options.native()?,
            |name, dimensions| files.resolve(name, dimensions, maximum_frames),
        )
        .map(|inner| SegmentationDocument { inner })
        .map_err(error)
    }
}
