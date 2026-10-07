//! Corpus re-estimation with annealing and ordered, independent worker statistics.
pub use liblrhsmm_rs::DurationMode;
use liblrhsmm_rs::{
    Dataset, GeometricOptions, HsmmOptions, Model, ModelError, ModelStatistics, Observation,
    Segmentation, UpdateOptions,
};

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub iterations: usize,
    pub duration_mode: DurationMode,
    pub hsmm: HsmmOptions,
    pub geometric: GeometricOptions,
    pub termination_threshold: f32,
    pub deterministic_annealing: bool,
    pub mean_frame_likelihood: bool,
    pub workers: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            iterations: 1,
            duration_mode: DurationMode::Normal,
            hsmm: HsmmOptions::default(),
            geometric: GeometricOptions::default(),
            termination_threshold: 1.0,
            deterministic_annealing: false,
            mean_frame_likelihood: false,
            workers: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IterationReport {
    pub iteration: usize,
    pub temperature: f32,
    pub mean_log_likelihood: f32,
    /// One row per file, one entry per isolated group (or a single embedded sample).
    pub file_likelihoods: Vec<Vec<f32>>,
}
#[derive(Debug, Clone)]
pub struct TrainingResult {
    pub model: Model,
    pub iterations: Vec<IterationReport>,
}

/// Each dataset represents one file, with either one embedded sample or its
/// ordered isolated groups. Input model and datasets are never modified.
pub fn train(
    model: &Model,
    files: &[Dataset],
    options: Options,
) -> Result<TrainingResult, ModelError> {
    train_with_progress(model, files, options, |_| {})
}

pub fn train_with_progress(
    model: &Model,
    files: &[Dataset],
    options: Options,
    mut progress: impl FnMut(&IterationReport),
) -> Result<TrainingResult, ModelError> {
    model.validate()?;
    if options.iterations > i32::MAX as usize
        || options.workers == 0
        || !options.termination_threshold.is_finite()
    {
        return Err(ModelError(
            "invalid training iteration, worker or threshold setting",
        ));
    }
    let mut result = TrainingResult {
        model: model.clone(),
        iterations: Vec::new(),
    };
    if options.iterations == 0 {
        return Ok(result);
    }
    if files.is_empty() {
        return Err(ModelError("training requires input files"));
    }
    let files = prepare_files(files)?;
    let mut previous = 0.0;
    for iteration in 0..options.iterations {
        let mut current = options;
        let temperature = if options.deterministic_annealing {
            f64::from((iteration + 1) as f32 / options.iterations as f32).sqrt() as f32
        } else {
            1.0
        };
        current.hsmm.temperature = temperature;
        current.geometric.temperature = temperature;
        let mut statistics = ModelStatistics::from_model(&result.model)?;
        let rows = if options.workers == 1 || files.len() == 1 {
            files
                .iter()
                .map(|file| estimate_file(&mut statistics, &result.model, file, current))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            let estimates = parallel_estimates(&result.model, &files, current)?;
            let mut rows = Vec::new();
            for (local, likelihoods) in estimates {
                merge(&mut statistics, local);
                rows.push(likelihoods);
            }
            rows
        };
        let mut total = 0.0f32;
        for row in &rows {
            let mut file_mean = 0.0f32;
            for &value in row {
                file_mean += value / row.len() as f32;
            }
            total += file_mean;
        }
        let mean = total / files.len() as f32 / temperature;
        if !mean.is_finite() {
            return Err(ModelError("training likelihood is not finite"));
        }
        result.model.update(
            &statistics,
            UpdateOptions {
                duration_mode: options.duration_mode,
                ..UpdateOptions::default()
            },
        )?;
        let report = IterationReport {
            iteration,
            temperature,
            mean_log_likelihood: mean,
            file_likelihoods: rows,
        };
        progress(&report);
        result.iterations.push(report);
        if iteration > 0
            && options.termination_threshold > 0.0
            && mean < previous + options.termination_threshold
        {
            break;
        }
        previous = mean;
    }
    Ok(result)
}

struct PreparedFile<'a> {
    observations: &'a [Observation],
    segmentations: Vec<Segmentation>,
}

fn prepare_files(files: &[Dataset]) -> Result<Vec<PreparedFile<'_>>, ModelError> {
    files
        .iter()
        .map(|file| {
            if file.observations.is_empty() || file.observations.len() != file.segmentations.len() {
                return Err(ModelError("training requires paired samples in every file"));
            }
            let mut file = PreparedFile {
                observations: &file.observations,
                segmentations: file.segmentations.clone(),
            };
            for (observation, segmentation) in file.observations.iter().zip(&mut file.segmentations)
            {
                observation.validate()?;
                segmentation.validate()?;
                if observation.frames == 0 || segmentation.boundaries.is_empty() {
                    return Err(ModelError("training requires frames and states"));
                }
                for boundary in &mut segmentation.boundaries {
                    *boundary = (*boundary).min(observation.frames as i32);
                }
            }
            Ok(file)
        })
        .collect()
}

fn estimate_file(
    statistics: &mut ModelStatistics,
    model: &Model,
    file: &PreparedFile<'_>,
    options: Options,
) -> Result<Vec<f32>, ModelError> {
    file.observations
        .iter()
        .zip(&file.segmentations)
        .map(|(observation, segmentation)| {
            let mut likelihood = match options.duration_mode {
                DurationMode::Normal => {
                    statistics.estimate_hsmm(model, observation, segmentation, options.hsmm)?
                }
                DurationMode::Geometric => statistics.estimate_geometric(
                    model,
                    observation,
                    segmentation,
                    options.geometric,
                )?,
            };
            if options.mean_frame_likelihood {
                likelihood /= observation.frames as f32;
            }
            if !likelihood.is_finite() {
                return Err(ModelError("sample likelihood is not finite"));
            }
            Ok(likelihood)
        })
        .collect()
}

type FileEstimate = (ModelStatistics, Vec<f32>);
fn parallel_estimates(
    model: &Model,
    files: &[PreparedFile<'_>],
    options: Options,
) -> Result<Vec<FileEstimate>, ModelError> {
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        let chunk_size = files.len().div_ceil(options.workers.min(files.len()));
        for chunk in files.chunks(chunk_size) {
            let handle = std::thread::Builder::new()
                .spawn_scoped(scope, move || {
                    chunk
                        .iter()
                        .map(|file| {
                            let mut statistics = ModelStatistics::from_model(model)?;
                            let row = estimate_file(&mut statistics, model, file, options)?;
                            Ok((statistics, row))
                        })
                        .collect::<Result<Vec<_>, ModelError>>()
                })
                .map_err(|_| ModelError("training worker could not be started"));
            handles.push(handle);
        }
        // Join every started worker, even when another worker failed.
        let mut estimates = Vec::new();
        let mut failure = None;
        for handle in handles {
            let batch = handle.and_then(|handle| {
                handle
                    .join()
                    .map_err(|_| ModelError("training worker panicked"))?
            });
            match batch {
                Ok(batch) => estimates.extend(batch),
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(estimates),
        }
    })
}

// Every accumulator is constructed from the same immutable model. The update
// validates finite values after ordered reduction, including addition overflow.
fn merge(target: &mut ModelStatistics, source: ModelStatistics) {
    for (target, source) in target.durations.iter_mut().zip(source.durations) {
        target.value_sum += source.value_sum;
        target.squared_sum += source.squared_sum;
        target.occupancy += source.occupancy;
    }
    for (target, source) in target.streams.iter_mut().zip(source.streams) {
        for (target, source) in target.mixtures.iter_mut().zip(source.mixtures) {
            target.state_occupancy += source.state_occupancy;
            for (target, source) in target.weighted_sums.iter_mut().zip(source.weighted_sums) {
                *target += source;
            }
            for (target, source) in target
                .weighted_squared_sums
                .iter_mut()
                .zip(source.weighted_squared_sums)
            {
                *target += source;
            }
            for (target, source) in target
                .component_occupancies
                .iter_mut()
                .zip(source.component_occupancies)
            {
                *target += source;
            }
        }
    }
}
