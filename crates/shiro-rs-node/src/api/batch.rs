use super::{Audio, AudioOptions, DitherSequence, FeatureOptions, Features, Wave, error};
use crate::callbacks::Function;
use crate::{
    batch,
    features::{Energy, FeatureKind},
};
use napi::Error as BindingError;
use std::path::Path;
fn preset(value: u32) -> Result<batch::Preset, BindingError> {
    match value {
        0 => Ok(batch::Preset::Mfcc12Da16k),
        1 => Ok(batch::Preset::Mfcc12Dae16k),
        2 => Ok(batch::Preset::Plpcc12Da16k),
        _ => Err(error("invalid extraction preset")),
    }
}
pub fn batch_feature_options(value: u32) -> Result<FeatureOptions, BindingError> {
    let value = preset(value)?.feature_options();
    Ok(FeatureOptions {
        kind: match value.kind {
            FeatureKind::Mfcc => 0,
            FeatureKind::Mfbe => 1,
            FeatureKind::Plpcc => 2,
        },
        order: value.order,
        channels: value.channels,
        frame_length: value.frame_length,
        hop: value.hop,
        sample_rate_hz: value.sample_rate_hz,
        minimum_bandwidth_hz: value.minimum_bandwidth_hz,
        warp: value.warp,
        include_dc: value.include_dc,
        energy: match value.energy {
            None => 0,
            Some(Energy::Rms) => 1,
            Some(Energy::Decibels) => 2,
        },
        delta: value.delta,
        acceleration: value.acceleration,
    })
}
#[derive(Clone)]
pub struct BatchOptions {
    audio: AudioOptions,
    input_extension: String,
}
impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            audio: AudioOptions::default(),
            input_extension: batch::Options::default().input_extension,
        }
    }
}
impl BatchOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn audio(&self) -> AudioOptions {
        self.audio
    }
    pub fn set_audio(&mut self, value: &AudioOptions) {
        self.audio = *value;
    }
    pub fn input_extension(&self) -> String {
        self.input_extension.clone()
    }
    pub fn set_input_extension(&mut self, value: String) {
        self.input_extension = value;
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
impl BatchOptions {
    pub(super) fn native(&self) -> Result<batch::Options, BindingError> {
        Ok(batch::Options {
            audio: self.audio.native()?,
            input_extension: self.input_extension.clone(),
        })
    }
}
#[derive(Clone)]
pub struct BatchOutputs {
    pub raw: String,
    pub parameters: String,
    pub mfcc: Option<String>,
}
impl BatchOutputs {
    pub fn new(raw: String, parameters: String, mfcc: Option<String>) -> Self {
        Self {
            raw,
            parameters,
            mfcc,
        }
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone)]
pub struct BatchExtraction {
    audio: Audio,
    features: Features,
    outputs: BatchOutputs,
}
impl BatchExtraction {
    pub fn new(audio: &Audio, features: &Features, outputs: &BatchOutputs) -> Self {
        Self {
            audio: audio.clone(),
            features: features.clone(),
            outputs: outputs.clone(),
        }
    }
    pub fn audio(&self) -> Audio {
        self.audio.clone()
    }
    pub fn set_audio(&mut self, value: &Audio) {
        self.audio = value.clone();
    }
    pub fn features(&self) -> Features {
        self.features.clone()
    }
    pub fn set_features(&mut self, value: &Features) {
        self.features = value.clone();
    }
    pub fn outputs(&self) -> BatchOutputs {
        self.outputs.clone()
    }
    pub fn set_outputs(&mut self, value: &BatchOutputs) {
        self.outputs = value.clone();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn extract_with_sequence(
        wave: &Wave,
        stem: &str,
        options: &BatchOptions,
        value: u32,
        sequence: &mut DitherSequence,
    ) -> Result<Self, BindingError> {
        Self::extract_native(wave, stem, options, value, || sequence.inner.next_uniform())
    }
    pub fn extract(
        wave: &Wave,
        stem: &str,
        options: &BatchOptions,
        value: u32,
        uniform: &Function,
    ) -> Result<Self, BindingError> {
        let mut failure = None;
        let result =
            Self::extract_native(wave, stem, options, value, || match uniform.call0(&()) {
                Ok(value) => value.as_f64().map(|value| value as f32).unwrap_or(f32::NAN),
                Err(value) => {
                    failure = Some(value);
                    f32::NAN
                }
            });
        if let Some(value) = failure {
            return Err(value);
        }
        result
    }
}
impl BatchExtraction {
    fn extract_native(
        wave: &Wave,
        stem: &str,
        options: &BatchOptions,
        value: u32,
        uniform: impl FnMut() -> f32,
    ) -> Result<Self, BindingError> {
        let preset = preset(value)?;
        let options = options.native()?;
        let (_, outputs) = batch::output_paths(Path::new(stem), &options, false).map_err(error)?;
        let outputs = BatchOutputs {
            raw: outputs
                .raw
                .into_os_string()
                .into_string()
                .map_err(|_| error("raw path is not UTF-8"))?,
            parameters: outputs
                .parameters
                .into_os_string()
                .into_string()
                .map_err(|_| error("parameter path is not UTF-8"))?,
            mfcc: None,
        };
        let (audio, features) =
            batch::prepare_native(wave.inner.clone(), options.audio, preset, uniform)
                .map_err(error)?;
        Ok(Self {
            audio: Audio { inner: audio },
            features: Features { inner: features },
            outputs,
        })
    }
}
