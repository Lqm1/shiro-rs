use super::models::collection;
use super::{Datasets, Model, error};
use crate::training::{self, IterationReport as NativeReport, TrainingResult as NativeResult};
use liblrhsmm_rs::{DurationMode, GeometricOptions, HsmmOptions, ModelError};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
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
    fn native(&self) -> Result<training::Options, JsValue> {
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

#[wasm_bindgen]
impl TrainingOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct FileLikelihoods {
    inner: Vec<f32>,
}
#[wasm_bindgen]
impl FileLikelihoods {
    #[wasm_bindgen(constructor)]
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
collection!(LikelihoodRows, FileLikelihoods, Vec<f32>);

#[wasm_bindgen]
#[derive(Clone)]
pub struct IterationReport {
    inner: NativeReport,
}
#[wasm_bindgen]
impl IterationReport {
    #[wasm_bindgen(constructor)]
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
    #[wasm_bindgen(getter)]
    pub fn iteration(&self) -> usize {
        self.inner.iteration
    }
    #[wasm_bindgen(setter)]
    pub fn set_iteration(&mut self, value: usize) {
        self.inner.iteration = value;
    }
    #[wasm_bindgen(getter)]
    pub fn temperature(&self) -> f32 {
        self.inner.temperature
    }
    #[wasm_bindgen(setter)]
    pub fn set_temperature(&mut self, value: f32) {
        self.inner.temperature = value;
    }
    #[wasm_bindgen(getter)]
    pub fn mean_log_likelihood(&self) -> f32 {
        self.inner.mean_log_likelihood
    }
    #[wasm_bindgen(setter)]
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
collection!(IterationReports, IterationReport, NativeReport);

#[wasm_bindgen]
#[derive(Clone)]
pub struct TrainingResult {
    inner: NativeResult,
}
#[wasm_bindgen]
impl TrainingResult {
    #[wasm_bindgen(constructor)]
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
    pub fn write_likelihood_csv(&self) -> Result<Vec<u8>, JsValue> {
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

#[wasm_bindgen]
impl Model {
    pub fn train(
        &self,
        files: &Datasets,
        options: &TrainingOptions,
    ) -> Result<TrainingResult, JsValue> {
        training::train(&self.inner, files.native(), options.native()?)
            .map(|inner| TrainingResult { inner })
            .map_err(error)
    }
    pub fn train_with_progress(
        &self,
        files: &Datasets,
        options: &TrainingOptions,
        progress: &js_sys::Function,
    ) -> Result<TrainingResult, JsValue> {
        let mut callback_error = None;
        let result =
            training::train_observed(&self.inner, files.native(), options.native()?, |report| {
                let owned: JsValue = IterationReport {
                    inner: report.clone(),
                }
                .into();
                match progress.call1(&JsValue::UNDEFINED, &owned) {
                    Ok(_) => Ok(()),
                    Err(value) => {
                        callback_error = Some(value);
                        Err(ModelError("training progress callback failed"))
                    }
                }
            });
        if let Some(value) = callback_error {
            return Err(value);
        }
        result.map(|inner| TrainingResult { inner }).map_err(error)
    }
}
