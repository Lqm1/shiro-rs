//! Model-shaped rawfloat observations and JSON state sequences.
use crate::{
    labels::{SegmentationDocument, State},
    rawfloat,
};
use liblrhsmm_rs::{Dataset, Jump, Model, Observation, Segmentation};
use std::{
    fs::File,
    io::{self, BufReader, Read},
};
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
pub fn dimensions(model: &Model) -> io::Result<Vec<usize>> {
    model.validate().map_err(invalid_error)?;
    if model.streams.is_empty() {
        return Err(invalid("model requires observation streams"));
    }
    model
        .streams
        .iter()
        .map(|stream| {
            let dimension = stream
                .mixtures
                .first()
                .ok_or_else(|| invalid("stream requires emission states"))?
                .dimensions;
            Ok(dimension)
        })
        .collect()
}
fn invalid_error(error: liblrhsmm_rs::ModelError) -> io::Error {
    invalid(error.to_string())
}
/// Deinterleave complete frames into time-major independent streams.
pub fn read_observation(
    reader: impl Read,
    dimensions: &[usize],
    maximum_frames: usize,
) -> io::Result<Observation> {
    if dimensions.is_empty()
        || dimensions
            .iter()
            .any(|&dimension| dimension == 0 || dimension > i32::MAX as usize)
        || maximum_frames > i32::MAX as usize
    {
        return Err(invalid("invalid observation dimensions or frame limit"));
    }
    let stride = dimensions
        .iter()
        .try_fold(0usize, |sum, &dimension| sum.checked_add(dimension))
        .ok_or_else(|| invalid("observation frame width overflows"))?;
    // A caller's upper bound need not itself fit the current address space.
    // Bound storage before reading, without rejecting a small actual input.
    let maximum_samples = stride
        .saturating_mul(maximum_frames)
        .min(isize::MAX as usize / size_of::<f32>());
    let values = rawfloat::read(reader, maximum_samples)?;
    if values.len() % stride != 0 {
        return Err(invalid(
            "rawfloat size does not match complete model frames",
        ));
    }
    let mut observation =
        Observation::new(values.len() / stride, dimensions).map_err(invalid_error)?;
    for (time, frame) in values.chunks_exact(stride).enumerate() {
        let mut offset = 0;
        for stream in &mut observation.streams {
            let dimension = stream.dimensions;
            stream.values[time * dimension..(time + 1) * dimension]
                .copy_from_slice(&frame[offset..offset + dimension]);
            offset += dimension;
        }
    }
    Ok(observation)
}
/// Preserve C's truncating frame-boundary conversion and mixed double/float
/// residual arithmetic. Explicit forward entries are omitted, fixing C's
/// uninitialized slots, before appending the single ordinary forward edge.
pub fn read_segmentation(states: &[State], model: &Model) -> io::Result<Segmentation> {
    let dimensions = dimensions(model)?;
    if states.len() > i32::MAX as usize {
        return Err(invalid("segment count exceeds i32"));
    }
    let mut segmentation = Segmentation::new(dimensions.len(), states.len());
    for (index, state) in states.iter().enumerate() {
        if !state.time.is_finite() || state.time < 0.0 || state.time > i32::MAX as f64 {
            return Err(invalid(
                "state boundary must be finite, nonnegative and fit i32",
            ));
        }
        segmentation.boundaries[index] = state.time as i32;
        let duration = state
            .duration
            .filter(|&duration| duration < model.durations.len() && duration <= i32::MAX as usize)
            .ok_or_else(|| invalid("duration state is missing or out of range"))?;
        segmentation.duration_states[index] = duration as i32;
        let outputs = state
            .outputs
            .as_ref()
            .filter(|outputs| outputs.len() == dimensions.len())
            .ok_or_else(|| invalid("output stream count does not match the model"))?;
        for (stream, &output) in outputs.iter().enumerate() {
            if output >= model.streams[stream].mixtures.len() || output > i32::MAX as usize {
                return Err(invalid("emission state is out of range"));
            }
            segmentation.output_states[stream][index] = output as i32;
        }
        let mut residual = 1.0f32;
        let mut total = 0.0f64;
        let mut outgoing = Vec::new();
        for jump in state.jumps.iter().flatten() {
            let delta = jump
                .get("d")
                .and_then(serde_json::Value::as_i64)
                .filter(|&delta| delta >= i32::MIN as i64 && delta <= i32::MAX as i64)
                .ok_or_else(|| invalid("jump offset must fit i32"))?;
            let probability = jump
                .get("p")
                .and_then(serde_json::Value::as_f64)
                .filter(|p| p.is_finite() && *p >= 0.0 && *p <= 1.0)
                .ok_or_else(|| invalid("jump probability must be between zero and one"))?;
            if delta == 1 {
                continue;
            }
            residual = (f64::from(residual) - probability) as f32;
            total += probability;
            outgoing.push(Jump {
                delta: delta as i32,
                probability: probability as f32,
            });
        }
        if total > 1.0 {
            return Err(invalid("extra jump probabilities exceed one"));
        }
        outgoing.push(Jump {
            delta: 1,
            probability: residual.max(0.0),
        });
        segmentation.outgoing[index] = outgoing;
    }
    segmentation.validate().map_err(invalid_error)?;
    Ok(segmentation)
}
/// Resolve feature filenames exactly as stored, relative to the process cwd.
pub fn load(
    document: &SegmentationDocument,
    model: &Model,
    maximum_frames: usize,
) -> io::Result<Dataset> {
    let dimensions = dimensions(model)?;
    let mut observations = Vec::new();
    let mut segmentations = Vec::new();
    for file in &document.files {
        let observation = read_observation(
            BufReader::new(File::open(&file.filename)?),
            &dimensions,
            maximum_frames,
        )?;
        let segmentation = read_segmentation(&file.states, model)?;
        observations.push(observation);
        segmentations.push(segmentation);
    }
    Ok(Dataset {
        observations,
        segmentations,
    })
}
