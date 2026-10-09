//! Initial equally spaced segmentation and original optional phone topologies.
use crate::labels::{PhoneMap, State};
use serde_json::{Map, Value, json};
use std::io;
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn jump(states: &mut [State], index: usize, delta: usize, probability: Option<f64>) {
    let jumps = states[index]
        .jumps
        .as_mut()
        .expect("initial states have jumps");
    jumps.push(match probability {
        Some(p) => json!({"d":delta,"p":p}),
        None => json!({"d":delta}),
    });
}
/// Expand indexed phones, add original skip edges, and round flat boundaries
/// with floor(x + 0.5), retaining probability assignment over all extra edges.
pub fn initial(phones: &[String], map: &PhoneMap, frames: usize) -> io::Result<Vec<State>> {
    if frames > i32::MAX as usize {
        return Err(invalid("frame count exceeds i32"));
    }
    let mut states: Vec<State> = Vec::new();
    for name in phones {
        let phone = map
            .phones
            .get(name)
            .ok_or_else(|| invalid(format!("phoneme {name} is not defined in the phone map")))?;
        let count = phone.states.len();
        let start = states.len();
        start
            .checked_add(count)
            .filter(|&total| total <= i32::MAX as usize)
            .ok_or_else(|| invalid("segment count exceeds i32"))?;
        if let Some(probability) = phone.attributes.get("pskip") {
            let probability = probability
                .as_f64()
                .filter(|p| p.is_finite() && *p >= 0.0 && *p <= 1.0)
                .ok_or_else(|| invalid("skip probability must be between zero and one"))?;
            if probability > 0.0 && start > 0 {
                jump(&mut states, start - 1, count + 1, Some(probability));
            }
        }
        states
            .try_reserve(count)
            .map_err(|_| invalid("segment allocation failed"))?;
        for (index, source) in phone.states.iter().enumerate() {
            if source.duration > i32::MAX as usize
                || source
                    .outputs
                    .iter()
                    .any(|&value| value > i32::MAX as usize)
            {
                return Err(invalid("state index exceeds i32"));
            }
            states.push(State {
                time: (states.len() + 1) as f64,
                duration: Some(source.duration),
                outputs: Some(source.outputs.clone()),
                jumps: Some(Vec::new()),
                metadata: vec![Value::String(name.clone()), Value::from(index)],
                attributes: Map::new(),
            });
        }
        match phone.attributes.get("topology").and_then(Value::as_str) {
            Some("type-b" | "type-c") => {
                let is_b = phone.attributes["topology"] == "type-b";
                for offset in 0..count.saturating_sub(2) {
                    jump(
                        &mut states,
                        start + offset,
                        if is_b { count - offset - 1 } else { 2 },
                        None,
                    );
                }
            }
            Some("skip-boundary") => {
                if start > 0 {
                    jump(&mut states, start - 1, 2, None);
                }
                // Match Lua's global #states - 1 index even for a one-state phone.
                if states.len() >= 2 {
                    let index = states.len() - 2;
                    jump(&mut states, index, 2, None);
                }
            }
            _ => {}
        }
    }
    if states.is_empty() {
        return Ok(states);
    }
    let flat_duration = frames as f64 / states.len() as f64;
    for state in &mut states {
        state.time = (state.time * flat_duration + 0.5).floor();
        let jumps = state.jumps.as_mut().expect("initial states have jumps");
        let assigned: f64 = jumps
            .iter()
            .filter_map(|jump| jump.get("p").and_then(Value::as_f64))
            .sum();
        if assigned > 1.0 {
            return Err(invalid("explicit jump probabilities exceed one"));
        }
        if !jumps.is_empty() {
            let average = 0.5 * (1.0 - assigned) / jumps.len() as f64;
            for jump in jumps {
                if jump.get("p").is_none() {
                    jump["p"] = Value::from(average);
                }
            }
        }
    }
    Ok(states)
}

/// Count complete rawfloat feature frames using the original mkseg dimension
/// limits. Byte length comes from native metadata or an in-memory file's length.
pub fn feature_frame_count(bytes: u64, dimensions: usize) -> io::Result<usize> {
    if dimensions == 0 || dimensions > i32::MAX as usize {
        return Err(invalid("frame size must be positive and fit i32"));
    }
    let frame_bytes = (dimensions as u64) * 4;
    if !bytes.is_multiple_of(frame_bytes) {
        return Err(invalid("feature size does not match the frame size"));
    }
    usize::try_from(bytes / frame_bytes).map_err(|_| invalid("frame count exceeds usize"))
}
