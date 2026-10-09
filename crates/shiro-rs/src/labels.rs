//! Timed labels and SHIRO's JSON segmentation interchange.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
};

/// Remove the last extension using either legacy path separator, then append
/// a literal suffix. A leading dot counts as an extension in the Lua tools.
pub fn output_path(filename: &str, suffix: &str) -> PathBuf {
    let separator = filename.rfind(['/', '\\']);
    let stem = match filename.rfind('.') {
        Some(dot) if separator.is_none_or(|separator| dot > separator) => &filename[..dot],
        _ => filename,
    };
    crate::index::append_suffix(Path::new(stem), suffix)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub start: f64,
    pub end: f64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneMap {
    #[serde(rename = "phone_map")]
    pub phones: BTreeMap<String, Phone>,
    #[serde(flatten)]
    pub attributes: Map<String, Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phone {
    pub states: Vec<PhoneState>,
    #[serde(flatten)]
    pub attributes: Map<String, Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneState {
    #[serde(rename = "dur")]
    pub duration: usize,
    #[serde(rename = "out")]
    pub outputs: Vec<usize>,
    #[serde(flatten)]
    pub attributes: Map<String, Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentationDocument {
    #[serde(rename = "file_list")]
    pub files: Vec<SegmentedFile>,
    #[serde(flatten)]
    pub attributes: Map<String, Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedFile {
    pub filename: String,
    pub states: Vec<State>,
    #[serde(flatten)]
    pub attributes: Map<String, Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub time: f64,
    #[serde(rename = "dur", default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<usize>,
    #[serde(rename = "out", default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<usize>>,
    #[serde(rename = "jmp", default, skip_serializing_if = "Option::is_none")]
    pub jumps: Option<Vec<Value>>,
    /// Keep additional metadata after the phoneme and local state index.
    #[serde(rename = "ext")]
    #[serde(default)]
    pub metadata: Vec<Value>,
    #[serde(flatten)]
    pub attributes: Map<String, Value>,
}
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn hop_valid(hop: f64) -> io::Result<()> {
    if !hop.is_finite() || hop <= 0.0 {
        return Err(invalid("hop time must be finite and positive"));
    }
    Ok(())
}

/// Parse tab-separated labels, or space-separated labels when tabs are absent.
/// Unlike the Lua parser, a blank or malformed row cannot truncate later data.
pub fn parse(text: &str) -> io::Result<Vec<Label>> {
    let mut labels = Vec::new();
    for (line, row) in text.lines().enumerate() {
        if row.trim().is_empty() {
            continue;
        }
        let fields: Vec<_> = if row.contains('\t') {
            row.split('\t').collect()
        } else {
            row.split_ascii_whitespace().collect()
        };
        let fail = || invalid(format!("invalid label at line {}", line + 1));
        if fields.len() < 3 || fields[2].is_empty() {
            return Err(fail());
        }
        let start = fields[0].trim().parse::<f64>().map_err(|_| fail())?;
        let end = fields[1].trim().parse::<f64>().map_err(|_| fail())?;
        if !start.is_finite() || !end.is_finite() || start < 0.0 || end < start {
            return Err(fail());
        }
        labels.push(Label {
            start,
            end,
            name: fields[2].to_owned(),
        });
    }
    Ok(labels)
}

/// Divide each timed phoneme evenly among its mapped states, using Lua's
/// binary64 arithmetic order and ceiling at frame boundaries.
pub fn to_states(labels: &[Label], map: &PhoneMap, hop: f64) -> io::Result<Vec<State>> {
    hop_valid(hop)?;
    let mut states = Vec::new();
    for label in labels {
        if !label.start.is_finite()
            || !label.end.is_finite()
            || label.start < 0.0
            || label.end < label.start
        {
            return Err(invalid("invalid label interval"));
        }
        let phone = map.phones.get(&label.name).ok_or_else(|| {
            invalid(format!(
                "phoneme {} is not defined in the phone map",
                label.name
            ))
        })?;
        states
            .try_reserve(phone.states.len())
            .map_err(|_| invalid("state allocation failed"))?;
        for (index, source) in phone.states.iter().enumerate() {
            let time = ((label.start
                + (label.end - label.start) * (index + 1) as f64 / phone.states.len() as f64)
                / hop)
                .ceil();
            if !time.is_finite() || time > i32::MAX as f64 {
                return Err(invalid("label boundary exceeds the legacy frame range"));
            }
            if source.duration > i32::MAX as usize
                || source.outputs.is_empty()
                || source
                    .outputs
                    .iter()
                    .any(|&state| state > i32::MAX as usize)
            {
                return Err(invalid("phone state indices must fit the legacy format"));
            }
            states.push(State {
                time,
                duration: Some(source.duration),
                outputs: Some(source.outputs.clone()),
                jumps: Some(Vec::new()),
                metadata: vec![Value::String(label.name.clone()), Value::from(index)],
                attributes: Map::new(),
            });
        }
    }
    Ok(states)
}

/// Generate the original phoneme grouping. State output includes state rows
/// interleaved with phoneme rows, retaining the original `-s` behavior.
pub fn from_states(states: &[State], hop: f64, include_states: bool) -> io::Result<Vec<Label>> {
    hop_valid(hop)?;
    let mut labels = Vec::new();
    let mut phone_start = 0.0;
    let mut previous_end = 0.0;
    let mut previous: Option<(&str, f64)> = None;
    for state in states {
        let name = state
            .metadata
            .first()
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("state metadata requires a phoneme name"))?;
        let index = state
            .metadata
            .get(1)
            .and_then(Value::as_f64)
            .filter(|index| index.is_finite())
            .ok_or_else(|| invalid("state metadata requires a numeric state index"))?;
        let end = state.time * hop;
        if !state.time.is_finite() || !end.is_finite() || end < previous_end {
            return Err(invalid("state boundaries must be finite and nondecreasing"));
        }
        if let Some((old_name, old_index)) = previous
            && (index <= old_index || name != old_name)
        {
            labels.push(Label {
                start: phone_start,
                end: previous_end,
                name: old_name.to_owned(),
            });
            phone_start = previous_end;
        }
        if include_states {
            labels.push(Label {
                start: previous_end,
                end,
                name: index.to_string(),
            });
        }
        previous = Some((name, index));
        previous_end = end;
    }
    if let Some((name, _)) = previous {
        labels.push(Label {
            start: phone_start,
            end: previous_end,
            name: name.to_owned(),
        });
    }
    Ok(labels)
}

/// Tab-separated seconds with CRLF endings, using round-trip decimal values.
pub fn write(labels: &[Label], mut output: impl io::Write) -> io::Result<()> {
    for label in labels {
        if label.name.contains(['\r', '\n', '\t']) {
            return Err(invalid("label names cannot contain tabs or newlines"));
        }
        write!(output, "{}\t{}\t{}\r\n", label.start, label.end, label.name)?;
    }
    Ok(())
}
