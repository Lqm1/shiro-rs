use super::error;
use crate::features::{self, Energy, FeatureKind};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct FeatureOptions {
    pub kind: u32,
    pub order: usize,
    pub channels: usize,
    pub frame_length: usize,
    pub hop: f32,
    pub sample_rate_hz: f32,
    pub minimum_bandwidth_hz: f32,
    pub warp: f32,
    pub include_dc: bool,
    pub energy: u32,
    pub delta: bool,
    pub acceleration: bool,
}
impl Default for FeatureOptions {
    fn default() -> Self {
        let value = features::FeatureOptions::default();
        Self {
            kind: 0,
            order: value.order,
            channels: value.channels,
            frame_length: value.frame_length,
            hop: value.hop,
            sample_rate_hz: value.sample_rate_hz,
            minimum_bandwidth_hz: value.minimum_bandwidth_hz,
            warp: value.warp,
            include_dc: value.include_dc,
            energy: 0,
            delta: value.delta,
            acceleration: value.acceleration,
        }
    }
}
#[wasm_bindgen]
impl FeatureOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}
impl FeatureOptions {
    fn native(&self) -> Result<features::FeatureOptions, JsValue> {
        Ok(features::FeatureOptions {
            kind: match self.kind {
                0 => FeatureKind::Mfcc,
                1 => FeatureKind::Mfbe,
                2 => FeatureKind::Plpcc,
                _ => return Err(error("invalid feature kind")),
            },
            energy: match self.energy {
                0 => None,
                1 => Some(Energy::Rms),
                2 => Some(Energy::Decibels),
                _ => return Err(error("invalid energy mode")),
            },
            order: self.order,
            channels: self.channels,
            frame_length: self.frame_length,
            hop: self.hop,
            sample_rate_hz: self.sample_rate_hz,
            minimum_bandwidth_hz: self.minimum_bandwidth_hz,
            warp: self.warp,
            include_dc: self.include_dc,
            delta: self.delta,
            acceleration: self.acceleration,
        })
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Features {
    pub(super) inner: features::Features,
}
#[wasm_bindgen]
impl Features {
    #[wasm_bindgen(constructor)]
    pub fn new(frames: usize, columns: usize, values: &[f32]) -> Self {
        Self {
            inner: features::Features {
                frames,
                columns,
                values: values.to_vec(),
            },
        }
    }
    pub fn extract(signal: &[f32], options: &FeatureOptions) -> Result<Self, JsValue> {
        features::extract(signal, options.native()?)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    #[wasm_bindgen(getter)]
    pub fn frames(&self) -> usize {
        self.inner.frames
    }
    #[wasm_bindgen(setter)]
    pub fn set_frames(&mut self, value: usize) {
        self.inner.frames = value;
    }
    #[wasm_bindgen(getter)]
    pub fn columns(&self) -> usize {
        self.inner.columns
    }
    #[wasm_bindgen(setter)]
    pub fn set_columns(&mut self, value: usize) {
        self.inner.columns = value;
    }
    pub fn values(&self) -> Vec<f32> {
        self.inner.values.clone()
    }
    pub fn set_values(&mut self, values: &[f32]) {
        self.inner.values = values.to_vec();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
