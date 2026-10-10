use super::error;
use crate::audio;
use crate::callbacks::Function;
use pyo3::PyErr as BindingError;
use shiro_rs::dsp::{
    resampling::{BoundaryPolicy, KernelPolicy},
    wave::{self, Encoding},
};
pub struct DitherSequence {
    pub(super) inner: audio::DitherSequence,
}
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
#[derive(Clone, Copy, Default)]
pub struct AudioOptions {
    pub normalize: bool,
    pub dither_level: f32,
    pub output_sample_rate: Option<u32>,
    pub boundary: u32,
    pub kernel: u32,
}
impl AudioOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}
impl AudioOptions {
    pub(super) fn native(&self) -> Result<audio::AudioOptions, BindingError> {
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
#[derive(Clone)]
pub struct Wave {
    pub(super) inner: wave::Wave<f32>,
}
impl Wave {
    pub fn new(
        sample_rate: u32,
        bits_per_sample: u16,
        channels: u16,
        encoding: u32,
        samples: &[f32],
    ) -> Result<Self, BindingError> {
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
    pub fn read(bytes: &[u8], maximum_frames: usize) -> Result<Self, BindingError> {
        wave::read(&mut std::io::Cursor::new(bytes), maximum_frames)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn sample_rate(&self) -> u32 {
        self.inner.sample_rate
    }
    pub fn set_sample_rate(&mut self, value: u32) {
        self.inner.sample_rate = value;
    }
    pub fn bits_per_sample(&self) -> u16 {
        self.inner.bits_per_sample
    }
    pub fn set_bits_per_sample(&mut self, value: u16) {
        self.inner.bits_per_sample = value;
    }
    pub fn channels(&self) -> u16 {
        self.inner.channels
    }
    pub fn set_channels(&mut self, value: u16) {
        self.inner.channels = value;
    }
    pub fn encoding(&self) -> u32 {
        match self.inner.encoding {
            Encoding::Pcm => 0,
            Encoding::Float => 1,
        }
    }
    pub fn set_encoding(&mut self, value: u32) -> Result<(), BindingError> {
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
fn encoding_value(value: u32) -> Result<Encoding, BindingError> {
    match value {
        0 => Ok(Encoding::Pcm),
        1 => Ok(Encoding::Float),
        _ => Err(error("invalid wave encoding")),
    }
}
#[derive(Clone)]
pub struct Audio {
    pub(super) inner: audio::Audio,
}
impl Audio {
    pub fn new(sample_rate: u32, samples: &[f32]) -> Self {
        Self {
            inner: audio::Audio {
                sample_rate,
                samples: samples.to_vec(),
            },
        }
    }
    pub fn sample_rate(&self) -> u32 {
        self.inner.sample_rate
    }
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
    ) -> Result<Self, BindingError> {
        audio::prepare(wave.inner.clone(), options.native()?, || {
            sequence.inner.next_uniform()
        })
        .map(|inner| Self { inner })
        .map_err(error)
    }
    pub fn prepare(
        wave: &Wave,
        options: &AudioOptions,
        uniform: &Function,
    ) -> Result<Self, BindingError> {
        let mut failure = None;
        let result = audio::prepare(wave.inner.clone(), options.native()?, || {
            match uniform.call0(&()) {
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
