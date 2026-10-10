use super::{Datasets, Model, error};
use crate::callbacks::Function;
use crate::training::{self, IterationReport as NativeReport, TrainingResult as NativeResult};
use napi::Error as BindingError;
use shiro_rs::hsmm::{DurationMode, GeometricOptions, HsmmOptions, ModelError};
#[derive(Clone, Copy)]
pub struct TrainingOptions {
    pub iterations: usize,
    pub duration_mode: u32,
    pub hsmm_temperature: f32,
    pub duration_weight: f32,
    pub state_radius: f32,
    pub duration_extra: usize,
    pub duration_extra_factor: f32,
    pub geometric_temperature: f32,
    pub pruning_slope: f32,
    pub termination_threshold: f32,
    pub deterministic_annealing: bool,
    pub mean_frame_likelihood: bool,
    pub workers: usize,
}
impl Default for TrainingOptions {
    fn default() -> Self {
        let options = training::Options::default();
        Self {
            iterations: options.iterations,
            duration_mode: 0,
            hsmm_temperature: options.hsmm.temperature,
            duration_weight: options.hsmm.duration_weight,
            state_radius: options.hsmm.state_radius,
            duration_extra: options.hsmm.duration_extra,
            duration_extra_factor: options.hsmm.duration_extra_factor,
            geometric_temperature: options.geometric.temperature,
            pruning_slope: options.geometric.pruning_slope,
            termination_threshold: options.termination_threshold,
            deterministic_annealing: options.deterministic_annealing,
            mean_frame_likelihood: options.mean_frame_likelihood,
            workers: options.workers,
        }
    }
}
impl TrainingOptions {
    pub(super) fn native(&self) -> Result<training::Options, BindingError> {
        Ok(training::Options {
            iterations: self.iterations,
            duration_mode: match self.duration_mode {
                0 => DurationMode::Normal,
                1 => DurationMode::Geometric,
                _ => return Err(error("invalid duration mode")),
            },
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
            termination_threshold: self.termination_threshold,
            deterministic_annealing: self.deterministic_annealing,
            mean_frame_likelihood: self.mean_frame_likelihood,
            workers: self.workers,
        })
    }
}
impl TrainingOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}
#[derive(Clone)]
pub struct FileLikelihoods {
    inner: Vec<f32>,
}
impl FileLikelihoods {
    pub fn new(values: &[f32]) -> Self {
        Self {
            inner: values.to_vec(),
        }
    }
    pub fn values(&self) -> Vec<f32> {
        self.inner.clone()
    }
    pub fn replace(&mut self, values: &[f32]) {
        self.inner = values.to_vec();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone, Default)]
pub struct LikelihoodRows {
    pub(super) inner: Vec<Vec<f32>>,
}
impl LikelihoodRows {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<FileLikelihoods> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| FileLikelihoods { inner })
    }
    pub fn push(&mut self, value: &FileLikelihoods) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &FileLikelihoods) -> Result<(), BindingError> {
        *self
            .inner
            .get_mut(index)
            .ok_or_else(|| error("collection index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone)]
pub struct IterationReport {
    inner: NativeReport,
}
impl IterationReport {
    pub fn new(
        iteration: usize,
        temperature: f32,
        mean_log_likelihood: f32,
        rows: &LikelihoodRows,
    ) -> Self {
        Self {
            inner: NativeReport {
                iteration,
                temperature,
                mean_log_likelihood,
                file_likelihoods: rows.inner.clone(),
            },
        }
    }
    pub fn iteration(&self) -> usize {
        self.inner.iteration
    }
    pub fn set_iteration(&mut self, value: usize) {
        self.inner.iteration = value;
    }
    pub fn temperature(&self) -> f32 {
        self.inner.temperature
    }
    pub fn set_temperature(&mut self, value: f32) {
        self.inner.temperature = value;
    }
    pub fn mean_log_likelihood(&self) -> f32 {
        self.inner.mean_log_likelihood
    }
    pub fn set_mean_log_likelihood(&mut self, value: f32) {
        self.inner.mean_log_likelihood = value;
    }
    pub fn file_likelihoods(&self) -> LikelihoodRows {
        LikelihoodRows {
            inner: self.inner.file_likelihoods.clone(),
        }
    }
    pub fn set_file_likelihoods(&mut self, rows: &LikelihoodRows) {
        self.inner.file_likelihoods = rows.inner.clone();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone, Default)]
pub struct IterationReports {
    pub(super) inner: Vec<NativeReport>,
}
impl IterationReports {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<IterationReport> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| IterationReport { inner })
    }
    pub fn push(&mut self, value: &IterationReport) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &IterationReport) -> Result<(), BindingError> {
        *self
            .inner
            .get_mut(index)
            .ok_or_else(|| error("collection index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone)]
pub struct TrainingResult {
    inner: NativeResult,
}
impl TrainingResult {
    pub fn new(model: &Model, iterations: &IterationReports) -> Self {
        Self {
            inner: NativeResult {
                model: model.inner.clone(),
                iterations: iterations.inner.clone(),
            },
        }
    }
    pub fn model(&self) -> Model {
        Model {
            inner: self.inner.model.clone(),
        }
    }
    pub fn set_model(&mut self, model: &Model) {
        self.inner.model = model.inner.clone();
    }
    pub fn iterations(&self) -> IterationReports {
        IterationReports {
            inner: self.inner.iterations.clone(),
        }
    }
    pub fn write_likelihood_csv(&self) -> Result<Vec<u8>, BindingError> {
        let mut bytes = Vec::new();
        self.inner.write_likelihood_csv(&mut bytes).map_err(error)?;
        Ok(bytes)
    }
    pub fn set_iterations(&mut self, iterations: &IterationReports) {
        self.inner.iterations = iterations.inner.clone();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
impl Model {
    pub fn train(
        &self,
        files: &Datasets,
        options: &TrainingOptions,
    ) -> Result<TrainingResult, BindingError> {
        training::train(&self.inner, files.native(), options.native()?)
            .map(|inner| TrainingResult { inner })
            .map_err(error)
    }
    pub fn train_with_progress(
        &self,
        files: &Datasets,
        options: &TrainingOptions,
        progress: &Function,
    ) -> Result<TrainingResult, BindingError> {
        let mut callback_error = None;
        let result = training::try_train_with_progress(
            &self.inner,
            files.native(),
            options.native()?,
            |report| {
                let owned = IterationReport {
                    inner: report.clone(),
                };
                match progress.call_report(owned) {
                    Ok(_) => Ok(()),
                    Err(value) => {
                        callback_error = Some(value);
                        Err(ModelError("training progress callback failed"))
                    }
                }
            },
        );
        if let Some(value) = callback_error {
            return Err(value);
        }
        result.map(|inner| TrainingResult { inner }).map_err(error)
    }
}
