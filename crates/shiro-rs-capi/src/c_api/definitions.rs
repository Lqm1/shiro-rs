//! Complete native model definitions, including arbitrary public field values.
use super::{
    ShiroRsBytes, ShiroRsModel,
    buffers::{input, release},
    range, result,
};
use crate::definition::{DurationConstraint, ModelDefinition, StreamDefinition};
use std::io;

/// Independent complete native definition; construction does not build a model.
pub struct ShiroRsModelDefinition {
    pub(super) value: ModelDefinition,
}
/// Every native stream-definition field, with target-width dimensions and states.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiroRsStreamDefinition {
    pub states: usize,
    pub dimensions: usize,
    pub mixtures: usize,
    pub weight: f32,
}
impl From<&StreamDefinition> for ShiroRsStreamDefinition {
    fn from(value: &StreamDefinition) -> Self {
        Self {
            states: value.states,
            dimensions: value.dimensions,
            mixtures: value.mixtures,
            weight: value.weight,
        }
    }
}
/// Complete native optional bounds; flags zero absent, one present.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShiroRsDurationConstraint {
    pub index: usize,
    pub has_minimum: u32,
    pub minimum: i32,
    pub has_maximum: u32,
    pub maximum: i32,
}
impl From<&DurationConstraint> for ShiroRsDurationConstraint {
    fn from(value: &DurationConstraint) -> Self {
        Self {
            index: value.index,
            has_minimum: u32::from(value.minimum.is_some()),
            minimum: value.minimum.unwrap_or(0),
            has_maximum: u32::from(value.maximum.is_some()),
            maximum: value.maximum.unwrap_or(0),
        }
    }
}
/// Every top-level definition field; collections retain original ordering.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShiroRsDefinitionInfo {
    pub duration_states: usize,
    pub streams: usize,
    pub duration_constraints: usize,
}

/// Construct all native public fields without imposing model-building validation.
/// Arbitrary target-width integers and every binary32 weight bit are retained.
/// # Safety
/// Descriptor arrays are initialized aligned readable ranges of the stated count;
/// empty arrays permit null. Output is independent aligned writable storage holding
/// no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_create(
    duration_states: usize,
    streams: *const ShiroRsStreamDefinition,
    stream_count: usize,
    constraints: *const ShiroRsDurationConstraint,
    constraint_count: usize,
    output: *mut *mut ShiroRsModelDefinition,
) -> u32 {
    for status in [
        range(streams, stream_count),
        range(constraints, constraint_count),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Complete readable ranges, including empty null arrays.
    let streams = unsafe { input(streams, stream_count) };
    // SAFETY: Complete readable constraint range.
    let constraints = unsafe { input(constraints, constraint_count) };
    if constraints
        .iter()
        .any(|value| value.has_minimum > 1 || value.has_maximum > 1)
    {
        return 2;
    }
    // SAFETY: Immutable complete descriptors and independent output.
    unsafe {
        result(output, || {
            let mut copied_streams = Vec::new();
            copied_streams
                .try_reserve_exact(stream_count)
                .map_err(io::Error::other)?;
            for value in streams {
                copied_streams.push(StreamDefinition {
                    states: value.states,
                    dimensions: value.dimensions,
                    mixtures: value.mixtures,
                    weight: value.weight,
                });
            }
            let mut copied_constraints = Vec::new();
            copied_constraints
                .try_reserve_exact(constraint_count)
                .map_err(io::Error::other)?;
            for value in constraints {
                copied_constraints.push(DurationConstraint {
                    index: value.index,
                    minimum: (value.has_minimum == 1).then_some(value.minimum),
                    maximum: (value.has_maximum == 1).then_some(value.maximum),
                });
            }
            Ok(Box::into_raw(Box::new(ShiroRsModelDefinition {
                value: ModelDefinition {
                    duration_states,
                    streams: copied_streams,
                    duration_constraints: copied_constraints,
                },
            })))
        })
    }
}
/// Retrieve every top-level scalar and collection length.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_get_info(
    definition: *const ShiroRsModelDefinition,
    output: *mut ShiroRsDefinitionInfo,
) -> u32 {
    if let Err(status) = range(definition, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(ShiroRsDefinitionInfo {
                duration_states: (*definition).value.duration_states,
                streams: (*definition).value.streams.len(),
                duration_constraints: (*definition).value.duration_constraints.len(),
            })
        })
    }
}
/// Copy every checked stream field without normalizing scalar values.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_get_stream(
    definition: *const ShiroRsModelDefinition,
    index: usize,
    output: *mut ShiroRsStreamDefinition,
) -> u32 {
    if let Err(status) = range(definition, 1) {
        return status;
    }
    // SAFETY: Live readable owner and checked collection index.
    let Some(value) = (unsafe { &(*definition).value.streams }).get(index) else {
        return 2;
    };
    // SAFETY: Immutable stream and independent output.
    unsafe { result(output, || Ok(value.into())) }
}
/// Copy every checked duration constraint, retaining optional bound presence.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_get_constraint(
    definition: *const ShiroRsModelDefinition,
    index: usize,
    output: *mut ShiroRsDurationConstraint,
) -> u32 {
    if let Err(status) = range(definition, 1) {
        return status;
    }
    // SAFETY: Live readable owner and checked collection index.
    let Some(value) = (unsafe { &(*definition).value.duration_constraints }).get(index) else {
        return 2;
    };
    // SAFETY: Immutable constraint and independent output.
    unsafe { result(output, || Ok(value.into())) }
}
/// Read the original native JSON schema, including defaults and optional bounds.
/// # Safety
/// Bytes is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_read_json(
    bytes: *const ShiroRsBytes,
    output: *mut *mut ShiroRsModelDefinition,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    // SAFETY: Live immutable bytes and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsModelDefinition {
                value: serde_json::from_slice(&(*bytes).values).map_err(io::Error::other)?,
            })))
        })
    }
}
/// Write the original native JSON schema. Nonfinite weights fail rather than
/// silently losing their bits through JSON null; typed getters retain every bit.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_write_json(
    definition: *const ShiroRsModelDefinition,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(definition, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            if (*definition)
                .value
                .streams
                .iter()
                .any(|value| !value.weight.is_finite())
            {
                return Err(io::Error::other(
                    "JSON requires finite stream weights; use typed definition getters",
                ));
            }
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: serde_json::to_vec(&(*definition).value).map_err(io::Error::other)?,
            })))
        })
    }
}
/// Build a complete independent model using the native definition validation.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_build(
    definition: *const ShiroRsModelDefinition,
    output: *mut *mut ShiroRsModel,
) -> u32 {
    if let Err(status) = range(definition, 1) {
        return status;
    }
    // SAFETY: Live immutable definition and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsModel {
                value: (*definition).value.build().map_err(io::Error::other)?,
            })))
        })
    }
}
/// Deep-copy every public native definition field into independent ownership.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_clone(
    definition: *const ShiroRsModelDefinition,
    output: *mut *mut ShiroRsModelDefinition,
) -> u32 {
    if let Err(status) = range(definition, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsModelDefinition {
                value: (*definition).value.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_definition_release(
    slot: *mut *mut ShiroRsModelDefinition,
) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}
