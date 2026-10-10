use super::{
    Audio, DitherSequence, Features, IterationReports, Labels, Model, ModelDefinition, PhoneMap,
    SegmentationDocument, Wave, error,
};
use crate::callbacks::Function;
use crate::{features::FeatureKind, utterances};
use pyo3::PyErr as BindingError;
#[derive(Clone, Copy)]
pub struct UtteranceOptions {
    pub utterances: usize,
    pub hop_seconds: f64,
    pub minimum_silence_seconds: f64,
    pub minimum_voicing_seconds: f64,
    pub iterations: usize,
}
impl Default for UtteranceOptions {
    fn default() -> Self {
        let value = utterances::Options::default();
        Self {
            utterances: value.utterances,
            hop_seconds: value.hop_seconds,
            minimum_silence_seconds: value.minimum_silence_seconds,
            minimum_voicing_seconds: value.minimum_voicing_seconds,
            iterations: value.iterations,
        }
    }
}
impl UtteranceOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}
impl UtteranceOptions {
    pub(super) fn native(&self) -> utterances::Options {
        utterances::Options {
            utterances: self.utterances,
            hop_seconds: self.hop_seconds,
            minimum_silence_seconds: self.minimum_silence_seconds,
            minimum_voicing_seconds: self.minimum_voicing_seconds,
            iterations: self.iterations,
        }
    }
}
#[derive(Clone)]
enum Source {
    Fresh,
    Initialized(shiro_rs::hsmm::Model),
    Trained(shiro_rs::hsmm::Model),
}
#[derive(Clone)]
pub struct UtteranceModelSource {
    inner: Source,
}
impl UtteranceModelSource {
    pub fn fresh() -> Self {
        Self {
            inner: Source::Fresh,
        }
    }
    pub fn initialized(model: &Model) -> Self {
        Self {
            inner: Source::Initialized(model.inner.clone()),
        }
    }
    pub fn trained(model: &Model) -> Self {
        Self {
            inner: Source::Trained(model.inner.clone()),
        }
    }
    pub fn kind(&self) -> u32 {
        match self.inner {
            Source::Fresh => 0,
            Source::Initialized(_) => 1,
            Source::Trained(_) => 2,
        }
    }
    pub fn model(&self) -> Option<Model> {
        match &self.inner {
            Source::Fresh => None,
            Source::Initialized(inner) | Source::Trained(inner) => Some(Model {
                inner: inner.clone(),
            }),
        }
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
impl UtteranceModelSource {
    pub(super) fn native(&self) -> utterances::ModelSource<'_> {
        match &self.inner {
            Source::Fresh => utterances::ModelSource::Fresh,
            Source::Initialized(model) => utterances::ModelSource::Initialized(model),
            Source::Trained(model) => utterances::ModelSource::Trained(model),
        }
    }
}
#[derive(Clone)]
pub struct SegmentedUtterances {
    inner: utterances::SegmentedUtterances,
}
impl SegmentedUtterances {
    pub fn new(
        model: &Model,
        phonemap: &PhoneMap,
        definition: &ModelDefinition,
        initial_segmentation: &SegmentationDocument,
        alignment: &SegmentationDocument,
    ) -> Self {
        Self {
            inner: utterances::SegmentedUtterances {
                phonemap: phonemap.inner.clone(),
                definition: definition.inner.clone(),
                phones: Vec::new(),
                initial_segmentation: initial_segmentation.inner.clone(),
                uninitialized_model: None,
                initialized_model: None,
                model: model.inner.clone(),
                iterations: Vec::new(),
                alignment: alignment.inner.clone(),
                labels: Vec::new(),
            },
        }
    }
    pub fn split_features(
        features: &Features,
        filename: &str,
        options: &UtteranceOptions,
        source: &UtteranceModelSource,
    ) -> Result<Self, BindingError> {
        utterances::split_features(&features.inner, filename, options.native(), source.native())
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn phones_json(&self) -> Result<String, BindingError> {
        serde_json::to_string(&self.inner.phones).map_err(error)
    }
    pub fn set_phones_json(&mut self, json: &str) -> Result<(), BindingError> {
        let phones = serde_json::from_str(json).map_err(error)?;
        self.inner.phones = phones;
        Ok(())
    }
    pub fn uninitialized_model(&self) -> Option<Model> {
        self.inner
            .uninitialized_model
            .clone()
            .map(|inner| Model { inner })
    }
    pub fn set_uninitialized_model(&mut self, value: &Model) {
        self.inner.uninitialized_model = Some(value.inner.clone());
    }
    pub fn clear_uninitialized_model(&mut self) {
        self.inner.uninitialized_model = None;
    }
    pub fn initialized_model(&self) -> Option<Model> {
        self.inner
            .initialized_model
            .clone()
            .map(|inner| Model { inner })
    }
    pub fn set_initialized_model(&mut self, value: &Model) {
        self.inner.initialized_model = Some(value.inner.clone());
    }
    pub fn clear_initialized_model(&mut self) {
        self.inner.initialized_model = None;
    }
}
#[derive(Clone)]
pub struct SegmentedWave {
    inner: utterances::SegmentedWave,
}
impl SegmentedWave {
    pub fn new(audio: &Audio, features: &Features, utterances: &SegmentedUtterances) -> Self {
        Self {
            inner: utterances::SegmentedWave {
                audio: audio.inner.clone(),
                features: features.inner.clone(),
                utterances: utterances.inner.clone(),
            },
        }
    }
    pub fn split_with_sequence(
        wave: &Wave,
        filename: &str,
        dimensions: usize,
        kind: u32,
        options: &UtteranceOptions,
        source: &UtteranceModelSource,
        sequence: &mut DitherSequence,
    ) -> Result<Self, BindingError> {
        utterances::split_wave(
            wave.inner.clone(),
            filename,
            dimensions,
            feature_kind(kind)?,
            options.native(),
            source.native(),
            || sequence.inner.next_uniform(),
        )
        .map(|inner| Self { inner })
        .map_err(error)
    }
    pub fn split(
        wave: &Wave,
        filename: &str,
        dimensions: usize,
        kind: u32,
        options: &UtteranceOptions,
        source: &UtteranceModelSource,
        uniform: &Function,
    ) -> Result<Self, BindingError> {
        let mut failure = None;
        let result = utterances::split_wave(
            wave.inner.clone(),
            filename,
            dimensions,
            feature_kind(kind)?,
            options.native(),
            source.native(),
            || match uniform.call0(&()) {
                Ok(value) => value.as_f64().map(|value| value as f32).unwrap_or(f32::NAN),
                Err(value) => {
                    failure = Some(value);
                    f32::NAN
                }
            },
        );
        if let Some(value) = failure {
            return Err(value);
        }
        result.map(|inner| Self { inner }).map_err(error)
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
fn feature_kind(kind: u32) -> Result<FeatureKind, BindingError> {
    match kind {
        0 => Ok(FeatureKind::Mfcc),
        1 => Ok(FeatureKind::Mfbe),
        2 => Ok(FeatureKind::Plpcc),
        _ => Err(error("invalid feature kind")),
    }
}
impl SegmentedUtterances {
    pub fn phonemap(&self) -> PhoneMap {
        PhoneMap {
            inner: self.inner.phonemap.clone(),
        }
    }
    pub fn set_phonemap(&mut self, value: &PhoneMap) {
        self.inner.phonemap = value.inner.clone();
    }
}
impl SegmentedUtterances {
    pub fn definition(&self) -> ModelDefinition {
        ModelDefinition {
            inner: self.inner.definition.clone(),
        }
    }
    pub fn set_definition(&mut self, value: &ModelDefinition) {
        self.inner.definition = value.inner.clone();
    }
}
impl SegmentedUtterances {
    pub fn initial_segmentation(&self) -> SegmentationDocument {
        SegmentationDocument {
            inner: self.inner.initial_segmentation.clone(),
        }
    }
    pub fn set_initial_segmentation(&mut self, value: &SegmentationDocument) {
        self.inner.initial_segmentation = value.inner.clone();
    }
}
impl SegmentedUtterances {
    pub fn model(&self) -> Model {
        Model {
            inner: self.inner.model.clone(),
        }
    }
    pub fn set_model(&mut self, value: &Model) {
        self.inner.model = value.inner.clone();
    }
}
impl SegmentedUtterances {
    pub fn iterations(&self) -> IterationReports {
        IterationReports {
            inner: self.inner.iterations.clone(),
        }
    }
    pub fn set_iterations(&mut self, value: &IterationReports) {
        self.inner.iterations = value.inner.clone();
    }
}
impl SegmentedUtterances {
    pub fn alignment(&self) -> SegmentationDocument {
        SegmentationDocument {
            inner: self.inner.alignment.clone(),
        }
    }
    pub fn set_alignment(&mut self, value: &SegmentationDocument) {
        self.inner.alignment = value.inner.clone();
    }
}
impl SegmentedUtterances {
    pub fn labels(&self) -> Labels {
        Labels {
            inner: self.inner.labels.clone(),
        }
    }
    pub fn set_labels(&mut self, value: &Labels) {
        self.inner.labels = value.inner.clone();
    }
}
impl SegmentedWave {
    pub fn audio(&self) -> Audio {
        Audio {
            inner: self.inner.audio.clone(),
        }
    }
    pub fn set_audio(&mut self, value: &Audio) {
        self.inner.audio = value.inner.clone();
    }
}
impl SegmentedWave {
    pub fn features(&self) -> Features {
        Features {
            inner: self.inner.features.clone(),
        }
    }
    pub fn set_features(&mut self, value: &Features) {
        self.inner.features = value.inner.clone();
    }
}
impl SegmentedWave {
    pub fn utterances(&self) -> SegmentedUtterances {
        SegmentedUtterances {
            inner: self.inner.utterances.clone(),
        }
    }
    pub fn set_utterances(&mut self, value: &SegmentedUtterances) {
        self.inner.utterances = value.inner.clone();
    }
}
