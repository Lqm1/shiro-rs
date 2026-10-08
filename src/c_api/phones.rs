//! Complete phone maps and native model/segmentation transformations.
use super::{ShiroRsBytes, ShiroRsStates, buffers::release, range, result};
use crate::{labels::PhoneMap, phonemap, segmentation};
use std::io;

/// Independent complete phone map, including flattened JSON attributes.
pub struct ShiroRsPhoneMap {
    pub(super) value: PhoneMap,
}

/// Native phone expansion counts and flag. Topology is a separate optional UTF-8
/// byte owner so an absent topology remains distinct from an empty string.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiroRsPhoneOptions {
    pub states_per_phone: usize,
    pub streams: usize,
    /// Zero disables weak skips; one enables them. Other codes are invalid.
    pub weak_skips: u32,
}

/// Copy the native phone expansion defaults. Default topology is absent.
/// # Safety
/// Output is independent aligned exclusively writable descriptor storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_phone_options_default(output: *mut ShiroRsPhoneOptions) -> u32 {
    // SAFETY: Independent writable descriptor under the caller contract.
    unsafe {
        result(output, || {
            let options = phonemap::Options::default();
            Ok(ShiroRsPhoneOptions {
                states_per_phone: options.states_per_phone,
                streams: options.streams,
                weak_skips: u32::from(options.weak_skips),
            })
        })
    }
}

/// Expand the original UTF-8 phone text using every native option. Null topology
/// means absent; a byte owner may contain any UTF-8 topology, including empty.
/// # Safety
/// Text and optional topology are live readable byte owners. Options are aligned
/// initialized readable storage. Output is independent aligned writable storage
/// holding no live owner on success. Failed output slots remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_phone_map_create(
    text: *const ShiroRsBytes,
    options: *const ShiroRsPhoneOptions,
    topology: *const ShiroRsBytes,
    output: *mut *mut ShiroRsPhoneMap,
) -> u32 {
    for status in [
        range(text, 1),
        range(options, 1),
        range(topology, usize::from(!topology.is_null())),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Checked initialized readable descriptor.
    let options = unsafe { options.read() };
    let weak_skips = match options.weak_skips {
        0 => false,
        1 => true,
        _ => return 2,
    };
    // SAFETY: Live immutable input owners and independent writable output slot.
    unsafe {
        result(output, || {
            let text = std::str::from_utf8(&(*text).values).map_err(io::Error::other)?;
            let topology = if topology.is_null() {
                None
            } else {
                Some(
                    std::str::from_utf8(&(*topology).values)
                        .map_err(io::Error::other)?
                        .to_owned(),
                )
            };
            let value = phonemap::create(
                text,
                &phonemap::Options {
                    states_per_phone: options.states_per_phone,
                    streams: options.streams,
                    topology,
                    weak_skips,
                },
            )?;
            Ok(Box::into_raw(Box::new(ShiroRsPhoneMap { value })))
        })
    }
}

/// Read the complete original JSON phone map, retaining all additional attributes.
/// # Safety
/// Input is a live readable byte owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_phone_map_read_json(
    bytes: *const ShiroRsBytes,
    output: *mut *mut ShiroRsPhoneMap,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            let value = serde_json::from_slice(&(*bytes).values).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsPhoneMap { value })))
        })
    }
}

/// Write the complete original JSON map into independent byte storage.
/// # Safety
/// Input is a live readable map owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_phone_map_write_json(
    map: *const ShiroRsPhoneMap,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(map, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            let values = serde_json::to_vec(&(*map).value).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Deep-copy every phone, state and additional JSON attribute.
/// # Safety
/// Input is a live readable map owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_phone_map_clone(
    map: *const ShiroRsPhoneMap,
    output: *mut *mut ShiroRsPhoneMap,
) -> u32 {
    if let Err(status) = range(map, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsPhoneMap {
                value: (*map).value.clone(),
            })))
        })
    }
}

/// Release a unique map and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live map owner
/// or null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_phone_map_release(slot: *mut *mut ShiroRsPhoneMap) -> u32 {
    // SAFETY: Unique ownership transfer under the public contract.
    unsafe { release(slot) }
}

/// Convert the complete map into original model-definition JSON using native
/// state counts and deterministic tied-duration constraint intersections.
/// # Safety
/// Input is a live readable map owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_phone_map_to_definition(
    map: *const ShiroRsPhoneMap,
    dimensions: usize,
    hop: f64,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(map, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            let definition = phonemap::to_definition(&(*map).value, dimensions, hop)?;
            let values = serde_json::to_vec(&definition).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Generate all initial states from an ordered JSON array of UTF-8 phone names.
/// Preserve native topology, skip edges, binary64 rounding and state metadata.
/// # Safety
/// Inputs are live readable owners; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmentation_initial(
    names: *const ShiroRsBytes,
    map: *const ShiroRsPhoneMap,
    frames: usize,
    output: *mut *mut ShiroRsStates,
) -> u32 {
    if let Err(status) = range(names, 1) {
        return status;
    }
    if let Err(status) = range(map, 1) {
        return status;
    }
    // SAFETY: Live immutable owners and independent output.
    unsafe {
        result(output, || {
            let names: Vec<String> =
                serde_json::from_slice(&(*names).values).map_err(io::Error::other)?;
            let value = segmentation::initial(&names, &(*map).value, frames)?;
            Ok(Box::into_raw(Box::new(ShiroRsStates { value })))
        })
    }
}
