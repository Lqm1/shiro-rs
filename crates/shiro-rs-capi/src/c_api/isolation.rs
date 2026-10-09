//! Complete standalone isolated groups with original positions and metadata.
use super::{
    ShiroRsModel, ShiroRsObservation, ShiroRsStates,
    buffers::{input, release},
    range, result,
};
use crate::dataset::{self, IsolatedGroup};
use std::io;

/// Independent complete groups in original order.
pub struct ShiroRsIsolatedGroups {
    pub(super) value: Vec<IsolatedGroup>,
}
/// Every field needed to construct a native group. Owners are borrowed only
/// during construction; the collection deep-copies observation and state data.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShiroRsIsolatedGroupInput {
    pub first_state: usize,
    pub first_frame: usize,
    pub observation: *const ShiroRsObservation,
    pub states: *const ShiroRsStates,
}
/// Both original target-width position fields of an isolated group.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShiroRsIsolatedGroupInfo {
    pub first_state: usize,
    pub first_frame: usize,
}

/// Construct arbitrary complete public native group fields in caller order.
/// Empty collections and repeated input owners are permitted.
/// # Safety
/// Inputs are aligned initialized readable descriptors and live readable owners.
/// Output is independent aligned writable storage holding no live owner on
/// success. Inputs and failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_isolated_groups_create(
    groups: *const ShiroRsIsolatedGroupInput,
    count: usize,
    output: *mut *mut ShiroRsIsolatedGroups,
) -> u32 {
    if let Err(status) = range(groups, count) {
        return status;
    }
    // SAFETY: Checked readable descriptor storage under the caller contract.
    let groups = unsafe { input(groups, count) };
    for group in groups {
        if let Err(status) = range(group.observation, 1) {
            return status;
        }
        if let Err(status) = range(group.states, 1) {
            return status;
        }
    }
    // SAFETY: All readable owners checked and output is independent.
    unsafe {
        result(output, || {
            let mut value = Vec::new();
            value.try_reserve_exact(count).map_err(io::Error::other)?;
            for group in groups {
                value.push(IsolatedGroup {
                    first_state: group.first_state,
                    first_frame: group.first_frame,
                    observation: (*group.observation).value.clone(),
                    states: (*group.states).value.clone(),
                });
            }
            Ok(Box::into_raw(Box::new(ShiroRsIsolatedGroups { value })))
        })
    }
}

/// Split using native phone identity, boundary capping and local jump filtering.
/// Preserve all groups, original positions, observation bits and state metadata.
/// # Safety
/// Inputs are live readable owners; output is independent aligned writable
/// storage holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_isolated_groups(
    model: *const ShiroRsModel,
    observation: *const ShiroRsObservation,
    states: *const ShiroRsStates,
    output: *mut *mut ShiroRsIsolatedGroups,
) -> u32 {
    for status in [range(model, 1), range(observation, 1), range(states, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable owners and independent output.
    unsafe {
        result(output, || {
            let value =
                dataset::isolated_groups(&(*model).value, &(*observation).value, &(*states).value)?;
            Ok(Box::into_raw(Box::new(ShiroRsIsolatedGroups { value })))
        })
    }
}

/// Retrieve the ordered group count.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_isolated_groups_length(
    groups: *const ShiroRsIsolatedGroups,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(groups, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe { result(output, || Ok((*groups).value.len())) }
}

/// Copy both original position fields; an invalid index retains output.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_isolated_groups_get_info(
    groups: *const ShiroRsIsolatedGroups,
    index: usize,
    output: *mut ShiroRsIsolatedGroupInfo,
) -> u32 {
    if let Err(status) = range(groups, 1) {
        return status;
    }
    // SAFETY: Live immutable owner.
    let Some(group) = (unsafe { &(*groups).value }).get(index) else {
        return 2;
    };
    // SAFETY: Independent output storage.
    unsafe {
        result(output, || {
            Ok(ShiroRsIsolatedGroupInfo {
                first_state: group.first_state,
                first_frame: group.first_frame,
            })
        })
    }
}

/// Snapshot the complete observation into independent ownership.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_isolated_groups_get_observation(
    groups: *const ShiroRsIsolatedGroups,
    index: usize,
    output: *mut *mut ShiroRsObservation,
) -> u32 {
    if let Err(status) = range(groups, 1) {
        return status;
    }
    // SAFETY: Live immutable owner.
    let Some(group) = (unsafe { &(*groups).value }).get(index) else {
        return 2;
    };
    // SAFETY: Independent output storage.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsObservation {
                value: group.observation.clone(),
            })))
        })
    }
}

/// Snapshot all states, including local times, jumps and extra metadata.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_isolated_groups_get_states(
    groups: *const ShiroRsIsolatedGroups,
    index: usize,
    output: *mut *mut ShiroRsStates,
) -> u32 {
    if let Err(status) = range(groups, 1) {
        return status;
    }
    // SAFETY: Live immutable owner.
    let Some(group) = (unsafe { &(*groups).value }).get(index) else {
        return 2;
    };
    // SAFETY: Independent output storage.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsStates {
                value: group.states.clone(),
            })))
        })
    }
}

/// Deep-copy every original position, complete observation and full state sequence.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_isolated_groups_clone(
    groups: *const ShiroRsIsolatedGroups,
    output: *mut *mut ShiroRsIsolatedGroups,
) -> u32 {
    if let Err(status) = range(groups, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsIsolatedGroups {
                value: (*groups).value.clone(),
            })))
        })
    }
}

/// Release a unique owner and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_isolated_groups_release(
    slot: *mut *mut ShiroRsIsolatedGroups,
) -> u32 {
    // SAFETY: Unique ownership transfer under public contract.
    unsafe { release(slot) }
}
