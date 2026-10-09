//! Utterance-level segmentation using the original wavsplit model workflow.
use crate::{
    alignment, audio, dataset,
    definition::ModelDefinition,
    features::{self, Energy, FeatureKind, FeatureOptions, Features},
    initialization,
    labels::{self, Label, PhoneMap, SegmentationDocument, SegmentedFile},
    phonemap, segmentation, training,
};
use liblrhsmm_rs::{Dataset, Model, Observation};
use serde_json::{Map, json};
use std::io;

/// Audio and feature artifacts alongside the utterance model and labels.
#[derive(Debug, Clone)]
pub struct SegmentedWave {
    pub audio: audio::Audio,
    pub features: Features,
    pub utterances: SegmentedUtterances,
}

/// Run the original wavsplit feature pipeline and model workflow in memory.
/// The random source is caller-controlled, including when reproducing a
/// platform's original dither sequence. No intermediate files are required.
pub fn split_wave(
    wave: ciglet_rs::wave::Wave<f32>,
    filename: &str,
    dimensions: usize,
    kind: FeatureKind,
    options: Options,
    source: ModelSource<'_>,
    uniform: impl FnMut() -> f32,
) -> io::Result<SegmentedWave> {
    let order = dimensions
        .checked_sub(1)
        .filter(|&order| order > 0)
        .ok_or_else(|| invalid("utterance features require at least two dimensions"))?;
    let feature_options = FeatureOptions {
        kind,
        order,
        hop: (options.hop_seconds * 16000.0) as f32,
        sample_rate_hz: 16000.0,
        energy: Some(Energy::Rms),
        ..FeatureOptions::default()
    };
    // Validate before allocating audio or consuming the caller's random source.
    features::extract(&[], feature_options).map_err(invalid)?;
    let audio = audio::prepare(
        wave,
        audio::AudioOptions {
            output_sample_rate: Some(16000),
            dither_level: 0.01,
            ..audio::AudioOptions::default()
        },
        uniform,
    )
    .map_err(invalid)?;
    let features = features::extract(&audio.samples, feature_options).map_err(invalid)?;
    let utterances = split_features(&features, filename, options, source)?;
    Ok(SegmentedWave {
        audio,
        features,
        utterances,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub utterances: usize,
    pub hop_seconds: f64,
    pub minimum_silence_seconds: f64,
    pub minimum_voicing_seconds: f64,
    pub iterations: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            utterances: 1,
            hop_seconds: 0.1,
            minimum_silence_seconds: 0.3,
            minimum_voicing_seconds: 0.3,
            iterations: 15,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ModelSource<'a> {
    #[default]
    Fresh,
    Initialized(&'a Model),
    Trained(&'a Model),
}

#[derive(Debug, Clone)]
pub struct SegmentedUtterances {
    pub phonemap: PhoneMap,
    pub definition: ModelDefinition,
    pub phones: Vec<String>,
    pub initial_segmentation: SegmentationDocument,
    pub uninitialized_model: Option<Model>,
    pub initialized_model: Option<Model>,
    pub model: Model,
    pub iterations: Vec<training::IterationReport>,
    pub alignment: SegmentationDocument,
    pub labels: Vec<Label>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

/// Align alternating silence/utterance states from a single-stream feature
/// matrix. Existing initialized models train; trained models skip estimation.
/// The returned stage artifacts retain the original intermediate data formats.
pub fn split_features(
    features: &Features,
    filename: &str,
    options: Options,
    source: ModelSource<'_>,
) -> io::Result<SegmentedUtterances> {
    let state_count = options
        .utterances
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or_else(|| invalid("utterance count exceeds the legacy state range"))?;
    if !options.hop_seconds.is_finite()
        || options.hop_seconds <= 0.0
        || !options.minimum_silence_seconds.is_finite()
        || !options.minimum_voicing_seconds.is_finite()
        || options.iterations > i32::MAX as usize
        || features.frames == 0
        || features.frames > i32::MAX as usize
        || features.columns == 0
        || features.columns > i32::MAX as usize
        || features.frames.checked_mul(features.columns) != Some(features.values.len())
        || features.values.iter().any(|value| !value.is_finite())
    {
        return Err(invalid("invalid utterance options or feature matrix"));
    }
    let extra = (10.0 / options.hop_seconds).floor();
    if extra > i32::MAX as f64 {
        return Err(invalid("duration search exceeds i32"));
    }
    let map: PhoneMap = serde_json::from_value(json!({"phone_map": {
        "sil": { "states": [{"dur": 0, "out": [0]}], "durfloor": [options.minimum_silence_seconds] },
        "utt": { "states": [{"dur": 1, "out": [1]}], "durfloor": [options.minimum_voicing_seconds] }
    }})).map_err(invalid)?;
    let definition = phonemap::to_definition(&map, features.columns, options.hop_seconds)?;
    let mut phones = Vec::new();
    phones.try_reserve_exact(state_count).map_err(invalid)?;
    phones.push("sil".to_owned());
    for _ in 0..options.utterances {
        phones.extend(["utt".to_owned(), "sil".to_owned()]);
    }
    let states = segmentation::initial(&phones, &map, features.frames)?;
    let document = SegmentationDocument {
        files: vec![SegmentedFile {
            filename: filename.to_owned(),
            states,
            attributes: Map::new(),
        }],
        attributes: Map::new(),
    };
    let observation = Observation {
        frames: features.frames,
        streams: vec![liblrhsmm_rs::ObservationStream {
            dimensions: features.columns,
            values: features.values.clone(),
        }],
    };
    let uninitialized_model = if matches!(source, ModelSource::Fresh) {
        Some(definition.build().map_err(invalid)?)
    } else {
        None
    };
    let start = match source {
        ModelSource::Fresh => uninitialized_model
            .as_ref()
            .expect("fresh model was constructed"),
        ModelSource::Initialized(model) | ModelSource::Trained(model) => model,
    };
    let data = Dataset {
        observations: vec![observation],
        segmentations: vec![dataset::read_segmentation(
            &document.files[0].states,
            start,
        )?],
    };
    let initialized_model = if matches!(source, ModelSource::Fresh) {
        Some(
            initialization::initialize(
                start,
                &data,
                initialization::Options {
                    flat_start: true,
                    globally_tied: true,
                    variance_floor_ratio: 1.0,
                },
            )
            .map_err(invalid)?,
        )
    } else {
        None
    };
    let start = initialized_model.as_ref().unwrap_or(start);
    let radius = (0.2 * options.utterances as f64).floor() as f32;
    let (model, iterations) = if matches!(source, ModelSource::Trained(_)) {
        (start.clone(), Vec::new())
    } else {
        let mut training_options = training::Options {
            iterations: options.iterations,
            deterministic_annealing: true,
            termination_threshold: 0.0,
            ..training::Options::default()
        };
        training_options.hsmm.state_radius = radius;
        training_options.hsmm.duration_extra = extra as usize;
        let result = training::train(start, std::slice::from_ref(&data), training_options)
            .map_err(invalid)?;
        (result.model, result.iterations)
    };
    let mut alignment_options = alignment::Options::default();
    alignment_options.hsmm.state_radius = radius;
    alignment_options.hsmm.duration_extra = extra as usize;
    let mut aligned_states = alignment::align_states(
        &model,
        &data.observations[0],
        &document.files[0].states,
        alignment_options,
    )?;
    let mut number = 0;
    for state in &mut aligned_states {
        if state.metadata.first().and_then(serde_json::Value::as_str) == Some("utt") {
            state.metadata[0] = json!(number.to_string());
            number += 1;
        }
    }
    let labels = labels::from_states(&aligned_states, options.hop_seconds, false)?;
    let alignment = SegmentationDocument {
        files: vec![SegmentedFile {
            filename: filename.to_owned(),
            states: aligned_states,
            attributes: Map::new(),
        }],
        attributes: Map::new(),
    };
    Ok(SegmentedUtterances {
        phonemap: map,
        definition,
        phones,
        initial_segmentation: document,
        uninitialized_model,
        initialized_model,
        model,
        iterations,
        alignment,
        labels,
    })
}
