//! WAV-to-raw processing shared by the native command and library callers.
use ciglet_rs::{
    Error,
    resampling::{self, BoundaryPolicy, KernelPolicy},
    wave::Wave,
};

/// The original CLI's implicit seed-one C-runtime random sequence.
/// Windows uses its 15-bit recurrence; Linux GNU uses degree-31 feedback.
/// This preserves existing dither results without linking either C runtime.
pub struct DitherSequence {
    state: DitherState,
}
enum DitherState {
    Windows(u32),
    LinuxGnu { words: [u32; 31], front: usize },
}
impl DitherSequence {
    pub fn windows() -> Self {
        Self {
            state: DitherState::Windows(1),
        }
    }
    pub fn linux_gnu() -> Self {
        let mut words = [0u32; 31];
        words[0] = 1;
        for index in 1..words.len() {
            words[index] = (u64::from(words[index - 1]) * 16_807 % 2_147_483_647) as u32;
        }
        let mut sequence = Self {
            state: DitherState::LinuxGnu { words, front: 3 },
        };
        for _ in 0..310 {
            sequence.next_integer();
        }
        sequence
    }
    fn next_integer(&mut self) -> (u32, u32) {
        match &mut self.state {
            DitherState::Windows(word) => {
                *word = word.wrapping_mul(214_013).wrapping_add(2_531_011);
                ((*word >> 16) & 32_767, 32_767)
            }
            DitherState::LinuxGnu { words, front } => {
                let rear = (*front + 28) % words.len();
                words[*front] = words[*front].wrapping_add(words[rear]);
                let output = words[*front] >> 1;
                *front = (*front + 1) % words.len();
                (output, 2_147_483_647)
            }
        }
    }
    pub fn next_uniform(&mut self) -> f32 {
        let (draw, maximum) = self.next_integer();
        draw as f32 / maximum as f32
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AudioOptions {
    pub normalize: bool,
    /// Positive levels enable uniform additive noise; nonpositive levels are off.
    pub dither_level: f32,
    pub output_sample_rate: Option<u32>,
    pub boundary: BoundaryPolicy,
    pub kernel: KernelPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

/// Normalize, add dither, then resample in the original order. Uniform draws
/// must be in [0, 1]. A caller-controlled source makes the arithmetic testable
/// without depending on platform-specific C rand sequences.
pub fn prepare(
    wave: Wave<f32>,
    options: AudioOptions,
    mut uniform: impl FnMut() -> f32,
) -> Result<Audio, Error> {
    if wave.sample_rate == 0
        || wave.sample_rate > i32::MAX as u32
        || options
            .output_sample_rate
            .is_some_and(|rate| rate == 0 || rate > i32::MAX as u32)
        || !options.dither_level.is_finite()
        || wave.samples.len() > i32::MAX as usize
        || wave.samples.iter().any(|value| !value.is_finite())
    {
        return Err(Error(
            "audio requires finite samples, finite dither and positive bounded sample rates",
        ));
    }
    let mut samples = wave.samples;
    if options.normalize {
        let peak = samples
            .iter()
            .fold(0.0f32, |maximum, value| maximum.max(value.abs()));
        if peak > 0.0 {
            for sample in &mut samples {
                *sample /= peak;
            }
        }
    }
    if options.dither_level > 0.0 {
        for sample in &mut samples {
            let draw = uniform();
            if !draw.is_finite() || !(0.0..=1.0).contains(&draw) {
                return Err(Error("audio dither draws must be in zero to one"));
            }
            let signed = ((f64::from(draw) - 0.5) * 2.0) as f32;
            *sample += signed * options.dither_level;
            if !sample.is_finite() {
                return Err(Error("audio dither overflows"));
            }
        }
    }
    let sample_rate = options.output_sample_rate.unwrap_or(wave.sample_rate);
    if sample_rate != wave.sample_rate {
        let ratio = sample_rate as f32 / wave.sample_rate as f32;
        samples =
            resampling::resample_with_kernel(&samples, ratio, options.boundary, options.kernel)?;
    }
    Ok(Audio {
        sample_rate,
        samples,
    })
}
