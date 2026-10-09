use super::error;
use crate::audio;
use ciglet_rs::{
    resampling::{BoundaryPolicy, KernelPolicy},
    wave::{self, Encoding},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DitherSequence {
    pub(super) inner: audio::DitherSequence,
}
#[wasm_bindgen]
impl DitherSequence {
    pub fn windows() -> Self {
        Self {
            inner: audio::DitherSequence::windows(),
        }
    }
    pub fn linux_gnu() -> Self {
        Self {
            inner: audio::DitherSequence::linux_gnu(),
        }
    }
    pub fn next_uniform(&mut self) -> f32 {
        self.inner.next_uniform()
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Default)]
pub struct AudioOptions {
    pub normalize: bool,
    pub dither_level: f32,
    pub output_sample_rate: Option<u32>,
    pub boundary: u32,
    pub kernel: u32,
}
#[wasm_bindgen]
impl AudioOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}
impl AudioOptions {
    fn native(&self) -> Result<audio::AudioOptions, JsValue> {
        Ok(audio::AudioOptions {
            normalize: self.normalize,
            dither_level: self.dither_level,
            output_sample_rate: self.output_sample_rate,
            boundary: match self.boundary {
                0 => BoundaryPolicy::IncludeFirst,
                1 => BoundaryPolicy::LegacySkipFirst,
                _ => return Err(error("invalid boundary policy")),
            },
            kernel: match self.kernel {
                0 => KernelPolicy::Stable,
                1 => KernelPolicy::Legacy,
                _ => return Err(error("invalid kernel policy")),
            },
        })
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Wave {
    pub(super) inner: wave::Wave<f32>,
}
#[wasm_bindgen]
impl Wave {
    #[wasm_bindgen(constructor)]
    pub fn new(
        sample_rate: u32,
        bits_per_sample: u16,
        channels: u16,
        encoding: u32,
        samples: &[f32],
    ) -> Result<Self, JsValue> {
        Ok(Self {
            inner: wave::Wave {
                sample_rate,
                bits_per_sample,
                channels,
                encoding: encoding_value(encoding)?,
                samples: samples.to_vec(),
            },
        })
    }
    pub fn read(bytes: &[u8], maximum_frames: usize) -> Result<Self, JsValue> {
        wave::read(&mut std::io::Cursor::new(bytes), maximum_frames)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    #[wasm_bindgen(getter)]
    pub fn sample_rate(&self) -> u32 {
        self.inner.sample_rate
    }
    #[wasm_bindgen(setter)]
    pub fn set_sample_rate(&mut self, value: u32) {
        self.inner.sample_rate = value;
    }
    #[wasm_bindgen(getter)]
    pub fn bits_per_sample(&self) -> u16 {
        self.inner.bits_per_sample
    }
    #[wasm_bindgen(setter)]
    pub fn set_bits_per_sample(&mut self, value: u16) {
        self.inner.bits_per_sample = value;
    }
    #[wasm_bindgen(getter)]
    pub fn channels(&self) -> u16 {
        self.inner.channels
    }
    #[wasm_bindgen(setter)]
    pub fn set_channels(&mut self, value: u16) {
        self.inner.channels = value;
    }
    #[wasm_bindgen(getter)]
    pub fn encoding(&self) -> u32 {
        match self.inner.encoding {
            Encoding::Pcm => 0,
            Encoding::Float => 1,
        }
    }
    #[wasm_bindgen(setter)]
    pub fn set_encoding(&mut self, value: u32) -> Result<(), JsValue> {
        self.inner.encoding = encoding_value(value)?;
        Ok(())
    }
    pub fn samples(&self) -> Vec<f32> {
        self.inner.samples.clone()
    }
    pub fn set_samples(&mut self, samples: &[f32]) {
        self.inner.samples = samples.to_vec();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
fn encoding_value(value: u32) -> Result<Encoding, JsValue> {
    match value {
        0 => Ok(Encoding::Pcm),
        1 => Ok(Encoding::Float),
        _ => Err(error("invalid wave encoding")),
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Audio {
    pub(super) inner: audio::Audio,
}
#[wasm_bindgen]
impl Audio {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: u32, samples: &[f32]) -> Self {
        Self {
            inner: audio::Audio {
                sample_rate,
                samples: samples.to_vec(),
            },
        }
    }
    #[wasm_bindgen(getter)]
    pub fn sample_rate(&self) -> u32 {
        self.inner.sample_rate
    }
    #[wasm_bindgen(setter)]
    pub fn set_sample_rate(&mut self, value: u32) {
        self.inner.sample_rate = value;
    }
    pub fn samples(&self) -> Vec<f32> {
        self.inner.samples.clone()
    }
    pub fn set_samples(&mut self, samples: &[f32]) {
        self.inner.samples = samples.to_vec();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn prepare_with_sequence(
        wave: &Wave,
        options: &AudioOptions,
        sequence: &mut DitherSequence,
    ) -> Result<Self, JsValue> {
        audio::prepare(wave.inner.clone(), options.native()?, || {
            sequence.inner.next_uniform()
        })
        .map(|inner| Self { inner })
        .map_err(error)
    }
    pub fn prepare(
        wave: &Wave,
        options: &AudioOptions,
        uniform: &js_sys::Function,
    ) -> Result<Self, JsValue> {
        let mut failure = None;
        let result = audio::prepare(wave.inner.clone(), options.native()?, || {
            match uniform.call0(&JsValue::UNDEFINED) {
                Ok(value) => value.as_f64().map(|value| value as f32).unwrap_or(f32::NAN),
                Err(value) => {
                    failure = Some(value);
                    f32::NAN
                }
            }
        });
        if let Some(value) = failure {
            return Err(value);
        }
        result.map(|inner| Self { inner }).map_err(error)
    }
}
