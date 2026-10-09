//! Full native state fields without routing binary64 times through JSON.
use super::{ShiroRsArrayUsize, ShiroRsBytes, ShiroRsStates, buffers::input, range, result};
use crate::labels::State;
use std::io;

/// Every native state field. Null outputs/jumps are absent, live empty values
/// are present. Metadata is a JSON array; attributes is a JSON object. They are
/// parsed independently without merging or overriding native fields.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShiroRsStateInput {
    pub time: f64,
    pub has_duration: u32,
    pub duration: usize,
    pub outputs: *const ShiroRsArrayUsize,
    pub jumps: *const ShiroRsBytes,
    pub metadata: *const ShiroRsBytes,
    pub attributes: *const ShiroRsBytes,
}
/// All typed scalars and optional presence flags for a checked native state.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShiroRsStateInfo {
    pub time: f64,
    pub has_duration: u32,
    pub duration: usize,
    pub has_outputs: u32,
    pub has_jumps: u32,
}

/// Construct complete states in order, retaining every binary64 time bit and
/// optional value. Construction does not perform alignment/training validation.
/// # Safety
/// Input is initialized aligned readable storage of count descriptors, with
/// live immutable participating owners. Empty input permits null. Output is
/// independent aligned writable storage holding no live owner on success.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_create(
    values: *const ShiroRsStateInput,
    count: usize,
    output: *mut *mut ShiroRsStates,
) -> u32 {
    if let Err(status) = range(values, count) {
        return status;
    }
    // SAFETY: Complete readable initialized descriptor range.
    let values = unsafe { input(values, count) };
    for value in values {
        if value.has_duration > 1 {
            return 2;
        }
        for status in [
            range(value.outputs, usize::from(!value.outputs.is_null())),
            range(value.jumps, usize::from(!value.jumps.is_null())),
            range(value.metadata, 1),
            range(value.attributes, 1),
        ] {
            if let Err(status) = status {
                return status;
            }
        }
    }
    // SAFETY: Live immutable descriptors and nested owners, independent output.
    unsafe {
        result(output, || {
            let mut states = Vec::new();
            states.try_reserve_exact(count).map_err(io::Error::other)?;
            for value in values {
                states.push(State {
                    time: value.time,
                    duration: (value.has_duration == 1).then_some(value.duration),
                    outputs: if value.outputs.is_null() {
                        None
                    } else {
                        Some((*value.outputs).values.clone())
                    },
                    jumps: if value.jumps.is_null() {
                        None
                    } else {
                        Some(
                            serde_json::from_slice(&(*value.jumps).values)
                                .map_err(io::Error::other)?,
                        )
                    },
                    metadata: serde_json::from_slice(&(*value.metadata).values)
                        .map_err(io::Error::other)?,
                    attributes: serde_json::from_slice(&(*value.attributes).values)
                        .map_err(io::Error::other)?,
                });
            }
            Ok(Box::into_raw(Box::new(ShiroRsStates { value: states })))
        })
    }
}

/// Retrieve the complete ordered state count.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_length(
    states: *const ShiroRsStates,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(states, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe { result(output, || Ok((*states).value.len())) }
}

/// Retrieve exact time, duration and optional presence flags. Absent duration
/// returns scalar zero, distinct from a present zero via its presence flag.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage.
/// Invalid index leaves output unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_get_info(
    states: *const ShiroRsStates,
    index: usize,
    output: *mut ShiroRsStateInfo,
) -> u32 {
    if let Err(status) = range(states, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and checked index.
    let Some(value) = (unsafe { &(*states).value }).get(index) else {
        return 2;
    };
    // SAFETY: Immutable state and independent output.
    unsafe {
        result(output, || {
            Ok(ShiroRsStateInfo {
                time: value.time,
                has_duration: u32::from(value.duration.is_some()),
                duration: value.duration.unwrap_or(0),
                has_outputs: u32::from(value.outputs.is_some()),
                has_jumps: u32::from(value.jumps.is_some()),
            })
        })
    }
}

/// Snapshot complete outputs. Absent outputs or invalid index return two;
/// present empty outputs produce an independent empty owner.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_get_outputs(
    states: *const ShiroRsStates,
    index: usize,
    output: *mut *mut ShiroRsArrayUsize,
) -> u32 {
    if let Err(status) = range(states, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and checked index.
    let Some(values) = (unsafe { &(*states).value })
        .get(index)
        .and_then(|value| value.outputs.as_ref())
    else {
        return 2;
    };
    // SAFETY: Immutable row and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayUsize {
                values: values.clone(),
            })))
        })
    }
}

/// Snapshot one complete native JSON field: zero jumps, one metadata, two
/// attributes. Each field is serialized independently, preserving reserved
/// attribute names without merging them into typed state fields. Absent jumps,
/// invalid index or invalid field code return two and retain failed output.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_get_json_field(
    states: *const ShiroRsStates,
    index: usize,
    field: u32,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(states, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and checked index.
    let Some(value) = (unsafe { &(*states).value }).get(index) else {
        return 2;
    };
    if field > 2 || (field == 0 && value.jumps.is_none()) {
        return 2;
    }
    // SAFETY: Immutable state and independent output.
    unsafe {
        result(output, || {
            let values = match field {
                0 => serde_json::to_vec(&value.jumps),
                1 => serde_json::to_vec(&value.metadata),
                _ => serde_json::to_vec(&value.attributes),
            }
            .map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}
