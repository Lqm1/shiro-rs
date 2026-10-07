//! Embedded and isolated phoneme alignment using the original inference modes.
use crate::{
    dataset,
    labels::{SegmentationDocument, State},
};
use liblrhsmm_rs::{
    EmissionPruning, GeometricOptions, HsmmOptions, Model, Observation, PreparationOptions,
    PreparedModel, StateOccurrence,
};
use std::{
    fs::File,
    io::{self, BufReader},
};

#[derive(Debug, Clone, Copy, Default)]
pub enum DurationMode {
    #[default]
    Explicit,
    Geometric,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub duration_mode: DurationMode,
    pub isolated: bool,
    pub hsmm: HsmmOptions,
    pub geometric: GeometricOptions,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

fn prepare<'a>(
    model: &'a Model,
    options: Options,
    duration_states: impl Iterator<Item = usize>,
) -> io::Result<PreparedModel<'a>> {
    let durations: Vec<_> = match options.duration_mode {
        DurationMode::Explicit => duration_states.collect(),
        DurationMode::Geometric => Vec::new(),
    };
    model
        .prepare_selected_durations(
            PreparationOptions {
                temperature: options.hsmm.temperature,
                duration_weight: options.hsmm.duration_weight,
                ..PreparationOptions::default()
            },
            &durations,
        )
        .map_err(invalid)
}

/// Align without changing the input states or observation.
pub fn align_states(
    model: &Model,
    observation: &Observation,
    states: &[State],
    options: Options,
) -> io::Result<Vec<State>> {
    let prepared = prepare(
        model,
        options,
        states.iter().filter_map(|state| state.duration),
    )?;
    align_prepared(model, &prepared, observation, states, options)
}

/// Load each rawfloat file and reuse prepared distributions across the corpus.
/// Filenames retain their process-relative meaning. Errors leave the input intact.
pub fn align_document(
    model: &Model,
    document: &SegmentationDocument,
    options: Options,
) -> io::Result<SegmentationDocument> {
    let dimensions = dataset::dimensions(model)?;
    let prepared = prepare(
        model,
        options,
        document
            .files
            .iter()
            .flat_map(|file| &file.states)
            .filter_map(|state| state.duration),
    )?;
    let mut result = document.clone();
    for file in &mut result.files {
        let observation = dataset::read_observation(
            BufReader::new(File::open(&file.filename)?),
            &dimensions,
            i32::MAX as usize,
        )?;
        file.states = align_prepared(model, &prepared, &observation, &file.states, options)?;
    }
    Ok(result)
}

fn infer(
    model: &Model,
    prepared: &PreparedModel<'_>,
    observation: &Observation,
    states: &[State],
    options: Options,
) -> io::Result<Vec<StateOccurrence>> {
    if observation.frames == 0 || states.is_empty() {
        return Err(invalid("alignment requires frames and states"));
    }
    let mut segmentation = dataset::read_segmentation(states, model)?;
    for boundary in &mut segmentation.boundaries {
        *boundary = (*boundary).min(observation.frames as i32);
    }
    match options.duration_mode {
        DurationMode::Explicit => {
            let emissions = prepared
                .emission_log_probabilities(
                    observation,
                    &segmentation,
                    options.hsmm.temperature,
                    EmissionPruning::AroundBoundaries {
                        radius: options.hsmm.state_radius,
                    },
                )
                .map_err(invalid)?;
            Ok(prepared
                .viterbi_hsmm(&segmentation, &emissions, options.hsmm)
                .map_err(invalid)?
                .occurrences)
        }
        DurationMode::Geometric => {
            let emissions = prepared
                .emission_log_probabilities(
                    observation,
                    &segmentation,
                    options.geometric.temperature,
                    EmissionPruning::Linear {
                        slope: options.geometric.pruning_slope,
                    },
                )
                .map_err(invalid)?;
            Ok(model
                .viterbi_geometric(&segmentation, &emissions, options.geometric)
                .map_err(invalid)?
                .boundaries
                .into_iter()
                .enumerate()
                .map(|(state, end)| StateOccurrence { state, end })
                .collect())
        }
    }
}

fn align_prepared(
    model: &Model,
    prepared: &PreparedModel<'_>,
    observation: &Observation,
    states: &[State],
    options: Options,
) -> io::Result<Vec<State>> {
    observation.validate().map_err(invalid)?;
    if observation.frames == 0 || states.is_empty() {
        return Err(invalid("alignment requires frames and states"));
    }
    if !options.isolated {
        let occurrences = infer(model, prepared, observation, states, options)?;
        return materialize(
            states,
            &occurrences,
            matches!(options.duration_mode, DurationMode::Geometric),
        );
    }
    let mut occurrences = Vec::new();
    for group in dataset::isolated_groups(model, observation, states)? {
        let aligned = infer(model, prepared, &group.observation, &group.states, options)?;
        occurrences.extend(aligned.into_iter().map(|item| StateOccurrence {
            state: group.first_state + item.state,
            end: group.first_frame + item.end,
        }));
    }
    materialize(states, &occurrences, false)
}
fn materialize(
    states: &[State],
    occurrences: &[StateOccurrence],
    retain_jumps: bool,
) -> io::Result<Vec<State>> {
    occurrences
        .iter()
        .map(|item| {
            let mut state = states
                .get(item.state)
                .ok_or_else(|| invalid("invalid inferred state"))?
                .clone();
            state.time = item.end as f64;
            if !retain_jumps {
                state.jumps = None;
            }
            Ok(state)
        })
        .collect()
}
