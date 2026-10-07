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

fn prepare(model: &Model, options: Options) -> io::Result<PreparedModel<'_>> {
    model
        .prepare(PreparationOptions {
            temperature: options.hsmm.temperature,
            duration_weight: options.hsmm.duration_weight,
            ..PreparationOptions::default()
        })
        .map_err(invalid)
}

/// Align without changing the input states or observation.
pub fn align_states(
    model: &Model,
    observation: &Observation,
    states: &[State],
    options: Options,
) -> io::Result<Vec<State>> {
    let prepared = prepare(model, options)?;
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
    let prepared = prepare(model, options)?;
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
    // Validate frame conversion before extracting any group from the observation.
    let segmentation = dataset::read_segmentation(states, model)?;
    let mut occurrences = Vec::new();
    let mut first = 0;
    let mut frame_start = 0;
    while first < states.len() {
        let (phone, mut previous) = identity(&states[first])?;
        let mut last = first + 1;
        while last < states.len() {
            let (next_phone, index) = identity(&states[last])?;
            if next_phone != phone || index <= previous {
                break;
            }
            previous = index;
            last += 1;
        }
        let frame_end = (segmentation.boundaries[last - 1] as usize).min(observation.frames);
        if frame_end <= frame_start {
            return Err(invalid(
                "isolated phoneme requires a positive frame interval",
            ));
        }
        let dimensions: Vec<_> = observation.streams.iter().map(|s| s.dimensions).collect();
        let mut local = Observation::new(frame_end - frame_start, &dimensions).map_err(invalid)?;
        for (source, target) in observation.streams.iter().zip(&mut local.streams) {
            target.values.copy_from_slice(
                &source.values[frame_start * source.dimensions..frame_end * source.dimensions],
            );
        }
        let mut group = states[first..last].to_vec();
        let count = group.len() as i64;
        for (index, state) in group.iter_mut().enumerate() {
            state.time = f64::from(segmentation.boundaries[first + index]) - frame_start as f64;
            // Filter using the source state, and compact accepted transitions.
            // Re-import probabilities from JSON to retain C's mixed precision.
            if let Some(jumps) = &mut state.jumps {
                jumps.retain(|jump| {
                    jump.get("d")
                        .and_then(serde_json::Value::as_i64)
                        .is_some_and(|delta| {
                            let destination = index as i64 + delta;
                            delta == 1 || (destination >= 0 && destination < count)
                        })
                });
            }
        }
        let aligned = infer(model, prepared, &local, &group, options)?;
        occurrences.extend(aligned.into_iter().map(|item| StateOccurrence {
            state: first + item.state,
            end: frame_start + item.end,
        }));
        frame_start = frame_end;
        first = last;
    }
    materialize(states, &occurrences, false)
}

fn identity(state: &State) -> io::Result<(&str, u64)> {
    let phone = state
        .metadata
        .first()
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| invalid("isolated alignment requires a phoneme in ext[0]"))?;
    let index = state
        .metadata
        .get(1)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| invalid("isolated alignment requires a state index in ext[1]"))?;
    Ok((phone, index))
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
