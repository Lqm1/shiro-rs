//! Complete timed labels, native conversions and original output paths.
use super::{
    ShiroRsBytes, ShiroRsPhoneMap, ShiroRsStates,
    buffers::{input, release},
    range, result,
};
use crate::labels::{self, Label};
use std::io;

/// Independent complete timed labels in original order.
pub struct ShiroRsLabels {
    pub(super) value: Vec<Label>,
}
/// Complete caller-supplied label. Name is a live readable UTF-8 byte owner.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShiroRsLabelInput {
    pub start: f64,
    pub end: f64,
    pub name: *const ShiroRsBytes,
}
/// Exact binary64 label times; names are retrieved as independent byte owners.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiroRsLabelInfo {
    pub start: f64,
    pub end: f64,
}

/// Copy all caller labels, preserving every time bit and full UTF-8 name.
/// Each native transformation retains its own validation; construction does not
/// reject intervals that native Label itself can represent.
/// # Safety
/// Inputs are aligned initialized readable descriptors and live readable names.
/// Output is independent aligned writable storage holding no live owner on
/// success. Inputs and failed output slots remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_create(
    values: *const ShiroRsLabelInput,
    count: usize,
    output: *mut *mut ShiroRsLabels,
) -> u32 {
    if let Err(status) = range(values, count) {
        return status;
    }
    // SAFETY: Checked descriptor range under the caller's readable contract.
    let values = unsafe { input(values, count) };
    for value in values {
        if let Err(status) = range(value.name, 1) {
            return status;
        }
    }
    // SAFETY: All name owners checked; output is independent.
    unsafe {
        result(output, || {
            let mut labels = Vec::new();
            labels.try_reserve_exact(count).map_err(io::Error::other)?;
            for value in values {
                let name = std::str::from_utf8(&(*value.name).values)
                    .map_err(io::Error::other)?
                    .to_owned();
                labels.push(Label {
                    start: value.start,
                    end: value.end,
                    name,
                });
            }
            Ok(Box::into_raw(Box::new(ShiroRsLabels { value: labels })))
        })
    }
}

/// Retrieve the complete ordered label count.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_length(
    labels: *const ShiroRsLabels,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(labels, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe { result(output, || Ok((*labels).value.len())) }
}

/// Copy both binary64 time fields of a label. Invalid index retains output.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_get_info(
    labels: *const ShiroRsLabels,
    index: usize,
    output: *mut ShiroRsLabelInfo,
) -> u32 {
    if let Err(status) = range(labels, 1) {
        return status;
    }
    // SAFETY: Live immutable input.
    let Some(label) = (unsafe { &(*labels).value }).get(index) else {
        return 2;
    };
    // SAFETY: Independent writable output.
    unsafe {
        result(output, || {
            Ok(ShiroRsLabelInfo {
                start: label.start,
                end: label.end,
            })
        })
    }
}

/// Copy the complete UTF-8 name into an independent byte owner.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_get_name(
    labels: *const ShiroRsLabels,
    index: usize,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(labels, 1) {
        return status;
    }
    // SAFETY: Live immutable input.
    let Some(label) = (unsafe { &(*labels).value }).get(index) else {
        return 2;
    };
    // SAFETY: Independent output slot.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: label.name.as_bytes().to_vec(),
            })))
        })
    }
}

/// Deep-copy every label, time bit and name.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_clone(
    labels: *const ShiroRsLabels,
    output: *mut *mut ShiroRsLabels,
) -> u32 {
    if let Err(status) = range(labels, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsLabels {
                value: (*labels).value.clone(),
            })))
        })
    }
}

/// Release a unique owner and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live label owner
/// or null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_release(slot: *mut *mut ShiroRsLabels) -> u32 {
    // SAFETY: Unique ownership transfer under public contract.
    unsafe { release(slot) }
}

/// Parse UTF-8 text with the unchanged native tab/space and blank-row semantics.
/// # Safety
/// Input is a live readable byte owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_parse(
    text: *const ShiroRsBytes,
    output: *mut *mut ShiroRsLabels,
) -> u32 {
    if let Err(status) = range(text, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            let text = std::str::from_utf8(&(*text).values).map_err(io::Error::other)?;
            let value = labels::parse(text)?;
            Ok(Box::into_raw(Box::new(ShiroRsLabels { value })))
        })
    }
}

/// Convert complete labels and phone map into all native states, retaining native
/// binary64 boundary arithmetic and state metadata.
/// # Safety
/// Inputs are live readable owners; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_to_states(
    labels: *const ShiroRsLabels,
    map: *const ShiroRsPhoneMap,
    hop: f64,
    output: *mut *mut ShiroRsStates,
) -> u32 {
    if let Err(status) = range(labels, 1) {
        return status;
    }
    if let Err(status) = range(map, 1) {
        return status;
    }
    // SAFETY: Live immutable inputs and independent output.
    unsafe {
        result(output, || {
            let value = labels::to_states(&(*labels).value, &(*map).value, hop)?;
            Ok(Box::into_raw(Box::new(ShiroRsStates { value })))
        })
    }
}

/// Convert all states into original phone grouping, optionally interleaving state
/// rows. include_states must be zero or one; hop retains native validation.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_from_states(
    states: *const ShiroRsStates,
    hop: f64,
    include_states: u32,
    output: *mut *mut ShiroRsLabels,
) -> u32 {
    if let Err(status) = range(states, 1) {
        return status;
    }
    let include_states = match include_states {
        0 => false,
        1 => true,
        _ => return 2,
    };
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            let value = labels::from_states(&(*states).value, hop, include_states)?;
            Ok(Box::into_raw(Box::new(ShiroRsLabels { value })))
        })
    }
}

/// Write every label using native round-trip decimals and CRLF endings.
/// Invalid names retain output; no partial buffer is published on failure.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_write_bytes(
    labels: *const ShiroRsLabels,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(labels, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            labels::write(&(*labels).value, &mut values)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Compute the original label output path, preserving both legacy separators,
/// leading-dot extension behavior and the literal suffix.
/// # Safety
/// Inputs are live readable UTF-8 byte owners; output is independent aligned
/// writable storage holding no live owner on success. Failed output unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_output_path(
    filename: *const ShiroRsBytes,
    suffix: *const ShiroRsBytes,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(filename, 1) {
        return status;
    }
    if let Err(status) = range(suffix, 1) {
        return status;
    }
    // SAFETY: Live immutable inputs and independent output.
    unsafe {
        result(output, || {
            let filename = std::str::from_utf8(&(*filename).values).map_err(io::Error::other)?;
            let suffix = std::str::from_utf8(&(*suffix).values).map_err(io::Error::other)?;
            let path = labels::output_path(filename, suffix);
            let path = path
                .to_str()
                .ok_or_else(|| io::Error::other("label output path is not UTF-8"))?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: path.as_bytes().to_vec(),
            })))
        })
    }
}
