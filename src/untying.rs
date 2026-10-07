//! Independent duration and emission distributions for each corpus occurrence.
use crate::labels::SegmentationDocument;
use liblrhsmm_rs::{Model, ModelBuilder, StreamBuilder};
use std::{
    fmt::Write as _,
    io::{self, Write},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assignment {
    pub state: usize,
    pub file: usize,
    pub segment: usize,
}

#[derive(Debug, Clone)]
pub struct UntiedModel {
    pub model: Model,
    pub segmentation: SegmentationDocument,
    pub assignments: Vec<Assignment>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

/// Clone distributions in file/state order and rewrite every model reference.
/// Times, jumps, metadata and unknown attributes are preserved. Observation
/// files are not opened, since untying depends only on model state references.
pub fn untie(model: &Model, document: &SegmentationDocument) -> io::Result<UntiedModel> {
    model.validate().map_err(invalid)?;
    let count = document
        .files
        .iter()
        .try_fold(0usize, |count, file| count.checked_add(file.states.len()))
        .filter(|&count| count <= i32::MAX as usize)
        .ok_or_else(|| invalid("untied state count exceeds the legacy format"))?;
    let mut builder = ModelBuilder::new(model.streams.len(), count).map_err(invalid)?;
    let mut streams = model
        .streams
        .iter()
        .map(|source| {
            let mut stream = StreamBuilder::new(count).map_err(invalid)?;
            stream.weight = source.weight;
            Ok(stream)
        })
        .collect::<io::Result<Vec<_>>>()?;
    let mut segmentation = document.clone();
    let mut assignments = Vec::new();
    assignments.try_reserve_exact(count).map_err(invalid)?;
    for (file_index, file) in segmentation.files.iter_mut().enumerate() {
        for (segment_index, segment) in file.states.iter_mut().enumerate() {
            let state = assignments.len();
            let duration = segment
                .duration
                .and_then(|index| model.durations.get(index))
                .ok_or_else(|| invalid("duration state is missing or out of range"))?;
            let outputs = segment
                .outputs
                .as_mut()
                .filter(|outputs| outputs.len() == streams.len())
                .ok_or_else(|| invalid("output stream count does not match the model"))?;
            builder.durations[state] = Some(duration.clone());
            for ((stream, source), output) in streams.iter_mut().zip(&model.streams).zip(outputs) {
                let distribution = source
                    .mixtures
                    .get(*output)
                    .ok_or_else(|| invalid("emission state is out of range"))?;
                stream.emissions[state] = Some(distribution.clone());
                *output = state;
            }
            segment.duration = Some(state);
            assignments.push(Assignment {
                state,
                file: file_index,
                segment: segment_index,
            });
        }
    }
    for (target, source) in builder.streams.iter_mut().zip(streams) {
        *target = Some(source.build().map_err(invalid)?);
    }
    Ok(UntiedModel {
        model: builder.build().map_err(invalid)?,
        segmentation,
        assignments,
    })
}

impl UntiedModel {
    /// Write the original global/file/local-index summary, with optional phone
    /// and local phone-state metadata. Validate all rows before writing bytes.
    pub fn write_summary(&self, mut writer: impl Write) -> io::Result<()> {
        let mut text = String::new();
        for assignment in &self.assignments {
            let state = self
                .segmentation
                .files
                .get(assignment.file)
                .and_then(|file| file.states.get(assignment.segment))
                .ok_or_else(|| invalid("summary assignment is out of range"))?;
            write!(
                text,
                "{} {} {}",
                assignment.state, assignment.file, assignment.segment
            )
            .map_err(invalid)?;
            if !state.metadata.is_empty() {
                let phone = state
                    .metadata
                    .first()
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| invalid("summary requires a phoneme string in ext[0]"))?;
                let index = state
                    .metadata
                    .get(1)
                    .and_then(serde_json::Value::as_f64)
                    .filter(|index| {
                        index.is_finite() && *index >= i32::MIN as f64 && *index <= i32::MAX as f64
                    })
                    .ok_or_else(|| invalid("summary requires an i32 state index in ext[1]"))?;
                write!(text, " {phone} {}", index as i32).map_err(invalid)?;
            }
            text.push('\n');
        }
        writer.write_all(text.as_bytes())
    }
}
