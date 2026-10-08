//! Complete model-shaped observations and lossless typed JSON state sequences.
use super::{
    ShiroRsBytes, ShiroRsModel,
    buffers::{input, release},
    range, result,
};
use crate::{dataset, labels::State};
use liblrhsmm_rs::Observation;
use std::io;

/// Independent native observation, including every stream and every frame.
pub struct ShiroRsObservation {
    pub(super) value: Observation,
}

/// Independent complete state sequence, including jumps and extra JSON metadata.
pub struct ShiroRsStates {
    pub(super) value: Vec<State>,
}

/// Deinterleave original rawfloat bytes with explicitly supplied stream widths.
/// The frame budget and complete-frame validation are those of the native loader.
/// # Safety
/// Owners are live values from this library. Dimensions are initialized aligned
/// readable storage of the stated length. Output is independent aligned writable
/// storage holding no live owner. No writable storage aliases participating inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_observation_read_rawfloat(
    bytes: *const ShiroRsBytes,
    dimensions: *const usize,
    stream_count: usize,
    maximum_frames: usize,
    output: *mut *mut ShiroRsObservation,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    if let Err(status) = range(dimensions, stream_count) {
        return status;
    }
    // SAFETY: Live input owners and validated borrowed dimensions; independent output.
    unsafe {
        result(output, || {
            let value = dataset::read_observation(
                (*bytes).values.as_slice(),
                input(dimensions, stream_count),
                maximum_frames,
            )?;
            Ok(Box::into_raw(Box::new(ShiroRsObservation { value })))
        })
    }
}

/// Deinterleave rawfloat bytes using all stream dimensions of a complete model.
/// # Safety
/// Inputs are live readable owners from this library. Output is independent
/// aligned writable storage holding no live owner and remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_observation_from_model_rawfloat(
    bytes: *const ShiroRsBytes,
    model: *const ShiroRsModel,
    maximum_frames: usize,
    output: *mut *mut ShiroRsObservation,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    if let Err(status) = range(model, 1) {
        return status;
    }
    // SAFETY: Live readable input owners and independent output slot.
    unsafe {
        result(output, || {
            let dimensions = dataset::dimensions(&(*model).value)?;
            let value =
                dataset::read_observation((*bytes).values.as_slice(), &dimensions, maximum_frames)?;
            Ok(Box::into_raw(Box::new(ShiroRsObservation { value })))
        })
    }
}

/// Serialize every observation field in the unchanged original MessagePack format.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner and remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_observation_write_bytes(
    observation: *const ShiroRsObservation,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(observation, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output slot.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            (*observation).value.write_to(&mut values)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Deep-copy all observation fields; the clone survives release of the source.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner and remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_observation_clone(
    observation: *const ShiroRsObservation,
    output: *mut *mut ShiroRsObservation,
) -> u32 {
    if let Err(status) = range(observation, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output slot.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsObservation {
                value: (*observation).value.clone(),
            })))
        })
    }
}

/// Release a unique observation and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is aligned independent initialized writable storage holding a unique
/// live owner from this library or null. Release requires exclusive access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_observation_release(slot: *mut *mut ShiroRsObservation) -> u32 {
    // SAFETY: Public contract transfers the unique allocation through its slot.
    unsafe { release(slot) }
}

/// Parse a complete JSON array of native states, retaining all additional fields.
/// # Safety
/// Input is a live readable bytes owner. Output is independent aligned writable
/// storage holding no live owner and remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_read_json(
    bytes: *const ShiroRsBytes,
    output: *mut *mut ShiroRsStates,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output slot.
    unsafe {
        result(output, || {
            let value = serde_json::from_slice(&(*bytes).values).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsStates { value })))
        })
    }
}

/// Serialize every typed state field and flattened additional JSON metadata.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner and remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_write_json(
    states: *const ShiroRsStates,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(states, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output slot.
    unsafe {
        result(output, || {
            let values = serde_json::to_vec(&(*states).value).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Deep-copy states, jumps, optional fields and all nested additional metadata.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner and remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_clone(
    states: *const ShiroRsStates,
    output: *mut *mut ShiroRsStates,
) -> u32 {
    if let Err(status) = range(states, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output slot.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsStates {
                value: (*states).value.clone(),
            })))
        })
    }
}

/// Release a unique state sequence and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is aligned independent initialized writable storage holding a unique
/// live owner from this library or null. Release requires exclusive access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_release(slot: *mut *mut ShiroRsStates) -> u32 {
    // SAFETY: Public contract transfers the unique allocation through its slot.
    unsafe { release(slot) }
}

/// Construct and serialize the original segmentation from every state field.
/// Uses the native boundary conversion and transition residual arithmetic, and
/// retains both legacy MessagePack array-header mismatches for C readers.
/// # Safety
/// Inputs are live readable owners from this library. Output is independent
/// aligned writable storage holding no live owner and remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_states_segmentation_bytes(
    states: *const ShiroRsStates,
    model: *const ShiroRsModel,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(states, 1) {
        return status;
    }
    if let Err(status) = range(model, 1) {
        return status;
    }
    // SAFETY: Live readable input owners and independent output slot.
    unsafe {
        result(output, || {
            let segmentation = dataset::read_segmentation(&(*states).value, &(*model).value)?;
            let mut values = Vec::new();
            segmentation.write_to(&mut values)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}
