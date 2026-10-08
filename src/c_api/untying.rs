//! Complete untied models, documents and arbitrary ordered assignments.
use super::{
    ShiroRsBytes, ShiroRsModel,
    buffers::{input, release},
    range, result,
};
use crate::{
    labels::SegmentationDocument,
    untying::{self, Assignment, UntiedModel},
};
use std::io;

/// Independent complete model, segmentation document and assignment table.
pub struct ShiroRsUntiedModel {
    pub(super) value: UntiedModel,
}
/// All native assignment fields in target-width integer representation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShiroRsAssignment {
    pub state: usize,
    pub file: usize,
    pub segment: usize,
}
impl From<Assignment> for ShiroRsAssignment {
    fn from(value: Assignment) -> Self {
        Self {
            state: value.state,
            file: value.file,
            segment: value.segment,
        }
    }
}

/// Construct all public native result fields without narrowing them to successful
/// untying results. Assignment validity is checked by native summary generation.
/// # Safety
/// Owners and assignment descriptors are live aligned initialized readable
/// storage. Output is independent aligned writable storage holding no live owner
/// on success. Inputs and failed output slots remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_create(
    model: *const ShiroRsModel,
    document: *const ShiroRsBytes,
    assignments: *const ShiroRsAssignment,
    count: usize,
    output: *mut *mut ShiroRsUntiedModel,
) -> u32 {
    for status in [
        range(model, 1),
        range(document, 1),
        range(assignments, count),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Checked readable owners/descriptors and independent output.
    unsafe {
        result(output, || {
            let segmentation =
                serde_json::from_slice(&(*document).values).map_err(io::Error::other)?;
            let mut copied = Vec::new();
            copied.try_reserve_exact(count).map_err(io::Error::other)?;
            copied.extend(input(assignments, count).iter().map(|a| Assignment {
                state: a.state,
                file: a.file,
                segment: a.segment,
            }));
            let value = UntiedModel {
                model: (*model).value.clone(),
                segmentation,
                assignments: copied,
            };
            Ok(Box::into_raw(Box::new(ShiroRsUntiedModel { value })))
        })
    }
}

/// Clone distributions in original file/state order, preserving every document
/// attribute and stream weight. Observation files are not opened.
/// # Safety
/// Inputs are live readable owners; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untie(
    model: *const ShiroRsModel,
    document: *const ShiroRsBytes,
    output: *mut *mut ShiroRsUntiedModel,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    if let Err(status) = range(document, 1) {
        return status;
    }
    // SAFETY: Live immutable inputs and independent output.
    unsafe {
        result(output, || {
            let document: SegmentationDocument =
                serde_json::from_slice(&(*document).values).map_err(io::Error::other)?;
            let value = untying::untie(&(*model).value, &document)?;
            Ok(Box::into_raw(Box::new(ShiroRsUntiedModel { value })))
        })
    }
}

/// Snapshot every model parameter into independent ownership.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_get_model(
    owner: *const ShiroRsUntiedModel,
    output: *mut *mut ShiroRsModel,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsModel {
                value: (*owner).value.model.clone(),
            })))
        })
    }
}

/// Snapshot the complete original segmentation document as JSON bytes.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_get_document(
    owner: *const ShiroRsUntiedModel,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            let values =
                serde_json::to_vec(&(*owner).value.segmentation).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Retrieve the complete ordered assignment count.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_length(
    owner: *const ShiroRsUntiedModel,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe { result(output, || Ok((*owner).value.assignments.len())) }
}

/// Copy all target-width fields of an ordered assignment.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
/// Invalid index retains output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_get_assignment(
    owner: *const ShiroRsUntiedModel,
    index: usize,
    output: *mut ShiroRsAssignment,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input.
    let Some(value) = (unsafe { &(*owner).value.assignments }).get(index) else {
        return 2;
    };
    // SAFETY: Independent output storage.
    unsafe { result(output, || Ok((*value).into())) }
}

/// Write the complete native summary into independent bytes. Native validation
/// covers all rows before output; invalid assignments/metadata retain output.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_summary_bytes(
    owner: *const ShiroRsUntiedModel,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            (*owner).value.write_summary(&mut values)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Deep-copy the complete model, document and assignments.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_clone(
    owner: *const ShiroRsUntiedModel,
    output: *mut *mut ShiroRsUntiedModel,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsUntiedModel {
                value: (*owner).value.clone(),
            })))
        })
    }
}

/// Release a unique owner and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_release(slot: *mut *mut ShiroRsUntiedModel) -> u32 {
    // SAFETY: Unique ownership transfer under public contract.
    unsafe { release(slot) }
}
