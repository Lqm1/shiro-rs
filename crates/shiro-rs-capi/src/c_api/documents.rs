//! Complete segmentation files and documents with independent typed state fields.
use super::{
    ShiroRsBytes, ShiroRsStates,
    buffers::{input, release},
    range, result,
};
use crate::labels::{SegmentationDocument, SegmentedFile};
use std::io;

/// Independent complete native file: filename, ordered states and attributes.
pub struct ShiroRsSegmentedFile {
    pub(super) value: SegmentedFile,
}
/// Independent complete native document: ordered files and attributes.
pub struct ShiroRsSegmentationDocument {
    pub(super) value: SegmentationDocument,
}

/// Copy every native file field. Filename is full UTF-8, including embedded NUL;
/// attributes is a JSON object decoded separately from the typed fields.
/// Construction does not impose alignment or training validation.
/// # Safety
/// Inputs are live immutable owners. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_file_create(
    filename: *const ShiroRsBytes,
    states: *const ShiroRsStates,
    attributes: *const ShiroRsBytes,
    output: *mut *mut ShiroRsSegmentedFile,
) -> u32 {
    for status in [range(filename, 1), range(states, 1), range(attributes, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable owners and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentedFile {
                value: SegmentedFile {
                    filename: std::str::from_utf8(&(*filename).values)
                        .map_err(io::Error::other)?
                        .to_owned(),
                    states: (*states).value.clone(),
                    attributes: serde_json::from_slice(&(*attributes).values)
                        .map_err(io::Error::other)?,
                },
            })))
        })
    }
}

/// Snapshot the complete UTF-8 filename into an independent byte owner.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_file_get_filename(
    file: *const ShiroRsSegmentedFile,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(file, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: (*file).value.filename.as_bytes().to_vec(),
            })))
        })
    }
}

/// Snapshot all ordered state fields without JSON conversion or time normalization.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_file_get_states(
    file: *const ShiroRsSegmentedFile,
    output: *mut *mut ShiroRsStates,
) -> u32 {
    if let Err(status) = range(file, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsStates {
                value: (*file).value.states.clone(),
            })))
        })
    }
}

/// Snapshot complete attributes as a JSON object without merging reserved keys.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_file_get_attributes(
    file: *const ShiroRsSegmentedFile,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(file, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: serde_json::to_vec(&(*file).value.attributes).map_err(io::Error::other)?,
            })))
        })
    }
}

/// Deep-copy all native file fields into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_file_clone(
    file: *const ShiroRsSegmentedFile,
    output: *mut *mut ShiroRsSegmentedFile,
) -> u32 {
    if let Err(status) = range(file, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentedFile {
                value: (*file).value.clone(),
            })))
        })
    }
}

/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_file_release(
    slot: *mut *mut ShiroRsSegmentedFile,
) -> u32 {
    // SAFETY: Unique ownership transfer under the public contract.
    unsafe { release(slot) }
}

/// Construct all ordered native files and attributes without workflow validation.
/// Repeated immutable file owners are permitted; attributes is a JSON object
/// decoded independently rather than merged into the typed file list.
/// # Safety
/// Files is an initialized aligned readable pointer range for count live
/// immutable owners; empty input permits null. Attributes is a live immutable
/// byte owner. Output is independent aligned writable storage holding no live
/// owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_document_create(
    files: *const *const ShiroRsSegmentedFile,
    count: usize,
    attributes: *const ShiroRsBytes,
    output: *mut *mut ShiroRsSegmentationDocument,
) -> u32 {
    for status in [range(files, count), range(attributes, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Complete initialized readable pointer range.
    let files = unsafe { input(files, count) };
    for &file in files {
        if let Err(status) = range(file, 1) {
            return status;
        }
    }
    // SAFETY: Live immutable file and attributes owners, independent output.
    unsafe {
        result(output, || {
            let attributes =
                serde_json::from_slice(&(*attributes).values).map_err(io::Error::other)?;
            let mut copied = Vec::new();
            copied.try_reserve_exact(count).map_err(io::Error::other)?;
            for &file in files {
                copied.push((*file).value.clone());
            }
            Ok(Box::into_raw(Box::new(ShiroRsSegmentationDocument {
                value: SegmentationDocument {
                    files: copied,
                    attributes,
                },
            })))
        })
    }
}

/// Retrieve the complete ordered file count.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_document_length(
    document: *const ShiroRsSegmentationDocument,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(document, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe { result(output, || Ok((*document).value.files.len())) }
}

/// Snapshot every checked file field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_document_get_file(
    document: *const ShiroRsSegmentationDocument,
    index: usize,
    output: *mut *mut ShiroRsSegmentedFile,
) -> u32 {
    if let Err(status) = range(document, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and checked index.
    let Some(value) = (unsafe { &(*document).value.files }).get(index) else {
        return 2;
    };
    // SAFETY: Immutable checked file and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentedFile {
                value: value.clone(),
            })))
        })
    }
}

/// Snapshot complete document attributes independently of the typed file list.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_document_get_attributes(
    document: *const ShiroRsSegmentationDocument,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(document, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: serde_json::to_vec(&(*document).value.attributes)
                    .map_err(io::Error::other)?,
            })))
        })
    }
}

/// Read the complete original native segmentation JSON schema.
/// # Safety
/// Input is a live immutable byte owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_document_read_json(
    bytes: *const ShiroRsBytes,
    output: *mut *mut ShiroRsSegmentationDocument,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentationDocument {
                value: serde_json::from_slice(&(*bytes).values).map_err(io::Error::other)?,
            })))
        })
    }
}

/// Write the original schema. Nonfinite state times fail rather than silently
/// becoming JSON null; typed file/state getters preserve their binary64 bits.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_document_write_json(
    document: *const ShiroRsSegmentationDocument,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(document, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            if (*document)
                .value
                .files
                .iter()
                .flat_map(|file| &file.states)
                .any(|state| !state.time.is_finite())
            {
                return Err(io::Error::other("JSON requires finite state times"));
            }
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: serde_json::to_vec(&(*document).value).map_err(io::Error::other)?,
            })))
        })
    }
}

/// Deep-copy every ordered file, state and attribute into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_document_clone(
    document: *const ShiroRsSegmentationDocument,
    output: *mut *mut ShiroRsSegmentationDocument,
) -> u32 {
    if let Err(status) = range(document, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentationDocument {
                value: (*document).value.clone(),
            })))
        })
    }
}

/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_document_release(
    slot: *mut *mut ShiroRsSegmentationDocument,
) -> u32 {
    // SAFETY: Unique ownership transfer under the public contract.
    unsafe { release(slot) }
}
