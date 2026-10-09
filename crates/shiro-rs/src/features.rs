//! Original xxcc feature extraction with reusable frame/FFT storage.
use ciglet_rs::{
    Error,
    cepstrum::{CepstralOptions, band_to_cepstrum},
    filterbank::{Compression, FilterBank, MelOptions},
    fourier::{Complex, Direction, FftPlan},
    window::{Precision, Window},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureKind {
    Mfcc,
    Mfbe,
    Plpcc,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Energy {
    Rms,
    Decibels,
}
#[derive(Debug, Clone, Copy)]
pub struct FeatureOptions {
    pub kind: FeatureKind,
    pub order: usize,
    pub channels: usize,
    pub frame_length: usize,
    /// Retains the original fractional spacing and truncated frame centers.
    pub hop: f32,
    pub sample_rate_hz: f32,
    pub minimum_bandwidth_hz: f32,
    pub warp: f32,
    pub include_dc: bool,
    pub energy: Option<Energy>,
    pub delta: bool,
    pub acceleration: bool,
}
impl Default for FeatureOptions {
    fn default() -> Self {
        Self {
            kind: FeatureKind::Mfcc,
            order: 12,
            channels: 36,
            frame_length: 1024,
            hop: 256.0,
            sample_rate_hz: 32000.0,
            minimum_bandwidth_hz: 400.0,
            warp: 1.0,
            include_dc: false,
            energy: None,
            delta: false,
            acceleration: false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Features {
    pub frames: usize,
    pub columns: usize,
    pub values: Vec<f32>,
}

pub fn extract(signal: &[f32], options: FeatureOptions) -> Result<Features, Error> {
    if options.order == 0
        || options.order >= i32::MAX as usize
        || options.channels == 0
        || options.channels > i32::MAX as usize
        || options.frame_length == 0
        || options.frame_length > i32::MAX as usize
        || signal.len() > i32::MAX as usize
        || !options.hop.is_finite()
        || options.hop < 1.0
        || !options.sample_rate_hz.is_finite()
        || options.sample_rate_hz <= 0.0
        || !options.minimum_bandwidth_hz.is_finite()
        || options.minimum_bandwidth_hz < 0.0
        || !options.warp.is_finite()
        || !(0.25..=4.0).contains(&options.warp)
        || signal.iter().any(|x| !x.is_finite())
    {
        return Err(Error("invalid feature extraction parameters or signal"));
    }
    let frame_count = signal.len() as f32 / options.hop;
    if frame_count >= 2_147_483_648.0 {
        return Err(Error("feature frame count exceeds the legacy range"));
    }
    let frames = frame_count as usize;
    let static_columns = options
        .order
        .checked_add(usize::from(options.include_dc))
        .and_then(|n| n.checked_add(usize::from(options.energy.is_some())))
        .ok_or(Error("feature dimension overflow"))?;
    let columns = static_columns
        .checked_mul(1 + usize::from(options.delta) + usize::from(options.acceleration))
        .ok_or(Error("feature dimension overflow"))?;
    let static_length = frames
        .checked_mul(static_columns)
        .ok_or(Error("feature matrix size overflow"))?;
    let output_length = frames
        .checked_mul(columns)
        .ok_or(Error("feature matrix size overflow"))?;
    if frames == 0 {
        return Ok(Features {
            frames,
            columns,
            values: Vec::new(),
        });
    }
    let channels = options.channels.max(options.order + 1);
    let fft_length = options
        .frame_length
        .checked_next_power_of_two()
        .ok_or(Error("feature transform length overflow"))?;
    let mut fft = FftPlan::<f32>::new(fft_length)?;
    let bins = fft_length / 2 + 1;
    let nyquist = options.sample_rate_hz / 2.0;
    let bank = if options.kind == FeatureKind::Plpcc {
        FilterBank::plp(bins, nyquist, channels)?
    } else {
        FilterBank::mel(
            bins,
            nyquist,
            channels,
            MelOptions {
                minimum_hz: 50.0,
                maximum_hz: nyquist,
                scale: options.warp,
                minimum_width_hz: options.minimum_bandwidth_hz,
            },
        )?
    };
    let window =
        Window::Blackman.generate_with::<f32>(options.frame_length, Precision::Accurate)?;
    let mut spectrum = filled(fft_length, Complex::new(0.0f32, 0.0))?;
    let mut magnitude = filled(bins, 0.0f32)?;
    let mut static_features = filled(static_length, 0.0f32)?;
    for frame in 0..frames {
        let center = options.hop * frame as f32;
        if center >= 2_147_483_648.0 {
            return Err(Error("feature center exceeds the legacy range"));
        }
        let origin = center as i64 - (options.frame_length / 2) as i64;
        spectrum.fill(Complex::new(0.0, 0.0));
        let mut energy = 0.0f32;
        for (index, &weight) in window.iter().enumerate() {
            let position = origin + index as i64;
            let sample = usize::try_from(position)
                .ok()
                .and_then(|i| signal.get(i))
                .copied()
                .unwrap_or(0.0);
            spectrum[index].re = sample * weight;
            energy += sample * sample * weight;
        }
        // Stored Blackman coefficients can make an endpoint slightly negative.
        // A squared energy cannot be negative; clamp this roundoff before sqrt.
        let rms = ((energy.max(0.0) / options.frame_length as f32) as f64).sqrt() as f32;
        fft.process(&mut spectrum, Direction::Forward)?;
        for (value, frequency) in magnitude.iter_mut().zip(&spectrum) {
            *value =
                ((frequency.re * frequency.re + frequency.im * frequency.im) as f64).sqrt() as f32;
        }
        let mut bands = bank.apply(
            &magnitude,
            if options.kind == FeatureKind::Plpcc {
                Compression::Plp
            } else {
                Compression::Logarithmic
            },
        )?;
        for value in &mut bands {
            *value = (*value).max(-15.0);
        }
        let row = &mut static_features[frame * static_columns..(frame + 1) * static_columns];
        if options.kind == FeatureKind::Mfbe {
            row[..options.order].copy_from_slice(&bands[..options.order]);
        } else {
            let cepstrum = band_to_cepstrum(
                &bands,
                CepstralOptions {
                    order: options.order,
                    include_dc: options.include_dc,
                },
            )?;
            row[..cepstrum.len()].copy_from_slice(&cepstrum);
        }
        if let Some(mode) = options.energy {
            row[static_columns - 1] = match mode {
                Energy::Rms => rms,
                Energy::Decibels => (20.0 * (rms as f64).log10()) as f32,
            };
        }
    }
    let mut output = filled(output_length, 0.0f32)?;
    dynamic(
        &static_features,
        frames,
        static_columns,
        &[1.0],
        &mut output,
        columns,
        0,
    );
    let mut offset = static_columns;
    if options.delta {
        dynamic(
            &static_features,
            frames,
            static_columns,
            &[-0.5, 0.0, 0.5],
            &mut output,
            columns,
            offset,
        );
        offset += static_columns;
    }
    if options.acceleration {
        dynamic(
            &static_features,
            frames,
            static_columns,
            &[0.25, 0.0, -0.5, 0.0, 0.25],
            &mut output,
            columns,
            offset,
        );
    }
    Ok(Features {
        frames,
        columns,
        values: output,
    })
}
fn dynamic(
    input: &[f32],
    frames: usize,
    dimensions: usize,
    kernel: &[f32],
    output: &mut [f32],
    columns: usize,
    offset: usize,
) {
    for frame in 0..frames {
        for dimension in 0..dimensions {
            let mut sum = 0.0f32;
            for (index, &weight) in kernel.iter().enumerate() {
                let source = frame as i64 + index as i64 - (kernel.len() / 2) as i64;
                if let Ok(source) = usize::try_from(source)
                    && source < frames
                {
                    sum += input[source * dimensions + dimension] * weight;
                }
            }
            output[frame * columns + offset + dimension] = sum;
        }
    }
}
fn filled<T: Clone>(length: usize, value: T) -> Result<Vec<T>, Error> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .map_err(|_| Error("feature allocation failed"))?;
    output.resize(length, value);
    Ok(output)
}
