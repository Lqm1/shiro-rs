//! SHIRO flat/aligned initialization without external processes.
use liblrhsmm_rs::{Dataset, Model, ModelError, ModelStatistics, UpdateOptions};
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub flat_start: bool,
    pub globally_tied: bool,
    pub variance_floor_ratio: f32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            flat_start: false,
            globally_tied: false,
            variance_floor_ratio: 0.1,
        }
    }
}
/// Return an initialized model, leaving input parameters and data unchanged.
/// Corpus-wide fallback duration uses all segment counts, correcting C's
/// overwritten count for multi-file corpora.
pub fn initialize(model: &Model, dataset: &Dataset, options: Options) -> Result<Model, ModelError> {
    if !options.variance_floor_ratio.is_finite() || options.variance_floor_ratio < 0.0 {
        return Err(ModelError(
            "variance floor ratio must be finite and nonnegative",
        ));
    }
    if dataset.observations.len() != dataset.segmentations.len() || dataset.observations.is_empty()
    {
        return Err(ModelError(
            "initialization requires paired observation/segmentation samples",
        ));
    }
    let mut statistics = ModelStatistics::from_model(model)?;
    let mut frames = 0u64;
    let mut segments = 0u64;
    for (observation, source) in dataset.observations.iter().zip(&dataset.segmentations) {
        if source.boundaries.is_empty() || observation.frames == 0 {
            return Err(ModelError("initialization requires frames and states"));
        }
        let mut segmentation = source.clone();
        for boundary in &mut segmentation.boundaries {
            *boundary = (*boundary).min(observation.frames as i32);
        }
        if options.flat_start {
            let duration = observation.frames as f32 / segmentation.boundaries.len() as f32;
            for (index, boundary) in segmentation.boundaries.iter_mut().enumerate() {
                *boundary = ((index + 1) as f64 * f64::from(duration)).floor() as i32;
            }
        }
        if options.globally_tied {
            for stream in &mut segmentation.output_states {
                stream.fill(0);
            }
        }
        statistics.collect_initialization(model, observation, &segmentation)?;
        frames = frames
            .checked_add(observation.frames as u64)
            .ok_or(ModelError("corpus frame count overflows"))?;
        segments = segments
            .checked_add(segmentation.boundaries.len() as u64)
            .ok_or(ModelError("corpus segment count overflows"))?;
    }
    let mut initialized = model.clone();
    initialized.update(&statistics, UpdateOptions::default())?;
    if options.globally_tied {
        for stream in &mut initialized.streams {
            let template = stream
                .mixtures
                .first()
                .ok_or(ModelError("stream requires emission states"))?
                .clone();
            for mixture in &mut stream.mixtures[1..] {
                *mixture = template.clone();
            }
        }
    }
    for stream in &mut initialized.streams {
        for mixture in &mut stream.mixtures {
            for (floor, &variance) in mixture.variance_floors.iter_mut().zip(&mixture.variances) {
                *floor = variance * options.variance_floor_ratio;
                if !floor.is_finite() {
                    return Err(ModelError("variance floor overflows"));
                }
            }
        }
    }
    let average = frames as f32 / segments as f32;
    for duration in &mut initialized.durations {
        if duration.mean == 0.0 {
            duration.mean = average;
            duration.variance = average * average;
            if !duration.variance.is_finite() {
                return Err(ModelError("fallback duration variance overflows"));
            }
        }
    }
    Ok(initialized)
}
