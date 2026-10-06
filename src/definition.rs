//! SHIRO's original JSON model definitions with clearer Rust field names.
use liblrhsmm_rs::{Duration, GaussianMixture, Model, ModelError, Stream};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDefinition {
    #[serde(rename = "ndurstate")]
    pub duration_states: usize,
    #[serde(rename = "streamdef")]
    pub streams: Vec<StreamDefinition>,
    #[serde(rename = "dur_attr", default)]
    pub duration_constraints: Vec<DurationConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDefinition {
    #[serde(rename = "nstate")]
    pub states: usize,
    #[serde(rename = "ndim")]
    pub dimensions: usize,
    #[serde(rename = "nmix", default = "one")]
    pub mixtures: usize,
    #[serde(default = "unit_weight")]
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationConstraint {
    pub index: usize,
    #[serde(rename = "floor", default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i32>,
    #[serde(rename = "ceil", default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i32>,
}
fn one() -> usize {
    1
}
fn unit_weight() -> f32 {
    1.0
}

impl ModelDefinition {
    /// Construct an uninitialized model with the original defaults.
    pub fn build(&self) -> Result<Model, ModelError> {
        if self.duration_states == 0 || self.duration_states > i32::MAX as usize {
            return Err(ModelError(
                "duration state count must be positive and fit i32",
            ));
        }
        if self.streams.is_empty() {
            return Err(ModelError("model requires at least one stream"));
        }
        // Validate before allocations or parameter mutation.
        for constraint in &self.duration_constraints {
            if constraint.index >= self.duration_states {
                return Err(ModelError("duration constraint index is out of range"));
            }
            if constraint
                .minimum
                .zip(constraint.maximum)
                .is_some_and(|(minimum, maximum)| minimum > 0 && maximum > 0 && minimum > maximum)
            {
                return Err(ModelError("duration minimum exceeds maximum"));
            }
        }
        let mut streams = Vec::new();
        for definition in &self.streams {
            if definition.states == 0 || definition.states > i32::MAX as usize {
                return Err(ModelError(
                    "emission state count must be positive and fit i32",
                ));
            }
            if !definition.weight.is_finite() || definition.weight < 0.0 {
                return Err(ModelError("stream weight must be finite and nonnegative"));
            }
            let template = GaussianMixture::new(definition.mixtures, definition.dimensions)?;
            streams.push(Stream {
                weight: definition.weight,
                mixtures: vec![template; definition.states],
            });
        }
        let mut durations = vec![Duration::default(); self.duration_states];
        for constraint in &self.duration_constraints {
            if let Some(minimum) = constraint.minimum {
                durations[constraint.index].minimum = minimum;
            }
            if let Some(maximum) = constraint.maximum {
                durations[constraint.index].maximum = maximum;
            }
        }
        Ok(Model { streams, durations })
    }
}
