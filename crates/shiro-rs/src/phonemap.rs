//! Phone-set expansion and conversion to the original SHIRO model definition.
use crate::{
    definition::{DurationConstraint, ModelDefinition, StreamDefinition},
    labels::{Phone, PhoneMap, PhoneState},
};
use serde_json::{Map, Value};
use std::{collections::BTreeMap, io};

#[derive(Debug, Clone)]
pub struct Options {
    pub states_per_phone: usize,
    pub streams: usize,
    pub topology: Option<String>,
    pub weak_skips: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            states_per_phone: 3,
            streams: 3,
            topology: None,
            weak_skips: false,
        }
    }
}
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
/// Expand phones in input order into disjoint duration/emission states.
pub fn create(text: &str, options: &Options) -> io::Result<PhoneMap> {
    if options.states_per_phone == 0
        || options.streams == 0
        || options.states_per_phone > i32::MAX as usize
        || options.streams > i32::MAX as usize
    {
        return Err(invalid(
            "state and stream counts must be positive and fit i32",
        ));
    }
    let mut phones = BTreeMap::new();
    let mut count = 0usize;
    for (line, row) in text.lines().enumerate() {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        if fields.is_empty() {
            continue;
        }
        if fields.len() % 2 == 0 {
            return Err(invalid(format!(
                "phone attributes require name/value pairs at line {}",
                line + 1
            )));
        }
        let name = fields[0];
        if phones.contains_key(name) {
            return Err(invalid(format!("duplicate phoneme {name}")));
        }
        let end = count
            .checked_add(options.states_per_phone)
            .filter(|&end| end <= i32::MAX as usize)
            .ok_or_else(|| invalid("phone state count exceeds i32"))?;
        let mut attributes = Map::new();
        if let Some(topology) = &options.topology {
            attributes.insert("topology".into(), Value::String(topology.clone()));
        }
        if options.weak_skips {
            attributes.insert("pskip".into(), Value::from(0.02));
        }
        for pair in fields[1..].chunks_exact(2) {
            if pair[0] != "durfloor" && pair[0] != "durceil" {
                continue;
            }
            let seconds = pair[1]
                .parse::<f64>()
                .map_err(|_| invalid(format!("invalid {} at line {}", pair[0], line + 1)))?;
            let per_state =
                ((seconds / options.states_per_phone as f64 * 1000.0) + 0.5).floor() / 1000.0;
            if !per_state.is_finite() {
                return Err(invalid("duration constraints must be finite"));
            }
            attributes.insert(
                pair[0].into(),
                Value::Array(vec![Value::from(per_state); options.states_per_phone]),
            );
        }
        let mut states = Vec::new();
        states
            .try_reserve_exact(options.states_per_phone)
            .map_err(|_| invalid("phone allocation failed"))?;
        for index in count..end {
            let mut outputs = Vec::new();
            outputs
                .try_reserve_exact(options.streams)
                .map_err(|_| invalid("stream allocation failed"))?;
            outputs.resize(options.streams, index);
            states.push(PhoneState {
                duration: index,
                outputs,
                attributes: Map::new(),
            });
        }
        phones.insert(name.to_owned(), Phone { states, attributes });
        count = end;
    }
    Ok(PhoneMap {
        phones,
        attributes: Map::new(),
    })
}
fn state_count(index: usize) -> io::Result<usize> {
    index
        .checked_add(1)
        .filter(|&count| count <= i32::MAX as usize)
        .ok_or_else(|| invalid("state index exceeds the legacy format"))
}
/// Compute independent per-stream state counts without mutating the map.
/// Shared-duration constraints intersect deterministically across phones.
pub fn to_definition(map: &PhoneMap, dimensions: usize, hop: f64) -> io::Result<ModelDefinition> {
    if dimensions == 0 || dimensions > i32::MAX as usize || !hop.is_finite() || hop <= 0.0 {
        return Err(invalid("dimensions and hop must be finite and positive"));
    }
    let mut durations = 0;
    let mut outputs: Vec<usize> = Vec::new();
    let mut constraints: BTreeMap<usize, DurationConstraint> = BTreeMap::new();
    for phone in map.phones.values() {
        for state in &phone.states {
            durations = durations.max(state_count(state.duration)?);
            if outputs.is_empty() {
                if state.outputs.is_empty() {
                    return Err(invalid("phone states require output streams"));
                }
                outputs.resize(state.outputs.len(), 0);
            }
            if outputs.len() != state.outputs.len() {
                return Err(invalid("inconsistent number of streams"));
            }
            for (maximum, &index) in outputs.iter_mut().zip(&state.outputs) {
                *maximum = (*maximum).max(state_count(index)?);
            }
        }
        for (key, is_floor) in [("durceil", false), ("durfloor", true)] {
            let Some(values) = phone.attributes.get(key) else {
                continue;
            };
            let values = values
                .as_array()
                .ok_or_else(|| invalid(format!("{key} requires an array")))?;
            if values.len() > phone.states.len() {
                return Err(invalid("duration constraint exceeds the phone state count"));
            }
            for (state, value) in phone.states.iter().zip(values) {
                let frames = (value
                    .as_f64()
                    .ok_or_else(|| invalid("duration constraint must be numeric"))?
                    / hop)
                    .ceil();
                if !frames.is_finite() || frames < i32::MIN as f64 || frames > i32::MAX as f64 {
                    return Err(invalid("duration constraint exceeds i32"));
                }
                let frames = frames as i32;
                let constraint = constraints
                    .entry(state.duration)
                    .or_insert(DurationConstraint {
                        index: state.duration,
                        minimum: None,
                        maximum: None,
                    });
                if is_floor {
                    constraint.minimum =
                        Some(constraint.minimum.map_or(frames, |old| old.max(frames)));
                } else {
                    constraint.maximum = Some(constraint.maximum.map_or(frames, |old| {
                        if old <= 0 && frames > 0 {
                            frames
                        } else if frames <= 0 && old > 0 {
                            old
                        } else {
                            old.min(frames)
                        }
                    }));
                }
            }
        }
    }
    for constraint in constraints.values() {
        if constraint
            .minimum
            .zip(constraint.maximum)
            .is_some_and(|(min, max)| min > 0 && max > 0 && min > max)
        {
            return Err(invalid("shared duration constraints contradict each other"));
        }
    }
    Ok(ModelDefinition {
        duration_states: durations,
        streams: outputs
            .into_iter()
            .map(|states| StreamDefinition {
                states,
                dimensions,
                mixtures: 1,
                weight: 1.0,
            })
            .collect(),
        duration_constraints: constraints.into_values().collect(),
    })
}
