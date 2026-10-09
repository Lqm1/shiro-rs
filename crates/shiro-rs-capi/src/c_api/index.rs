//! Complete index fields, ordered collections and native phoneme padding.
use super::{
    ShiroRsBytes, ShiroRsPath,
    buffers::{input, release},
    range, result,
};
use std::io::{self, Cursor};

/// Independent ordered UTF-8 strings, including empty and embedded NUL strings.
pub struct ShiroRsStrings {
    pub(super) values: Vec<String>,
}

/// Full native entry fields. Construction deep-copies both readable owners.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShiroRsIndexEntryInput {
    pub stem: *const ShiroRsPath,
    pub phonemes: *const ShiroRsStrings,
}

/// Independent ordered native index entries with every field retained.
pub struct ShiroRsIndexEntries {
    pub(super) values: Vec<crate::index::Entry>,
}

/// Deep-copy complete strings in input order. Repeated owners are permitted.
/// # Safety
/// Input is readable aligned pointer storage for count live immutable byte
/// owners; empty input permits null. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_strings_create(
    values: *const *const ShiroRsBytes,
    count: usize,
    output: *mut *mut ShiroRsStrings,
) -> u32 {
    if let Err(status) = range(values, count) {
        return status;
    }
    // SAFETY: Complete pointer array is readable; owner pointers checked below.
    let values = unsafe { input(values, count) };
    for &value in values {
        if let Err(status) = range(value, 1) {
            return status;
        }
    }
    // SAFETY: Live immutable owners and independent output.
    unsafe {
        result(output, || {
            let mut strings = Vec::new();
            strings.try_reserve_exact(count).map_err(io::Error::other)?;
            for &value in values {
                strings.push(
                    std::str::from_utf8(&(*value).values)
                        .map_err(io::Error::other)?
                        .to_owned(),
                );
            }
            Ok(Box::into_raw(Box::new(ShiroRsStrings { values: strings })))
        })
    }
}

/// Retrieve the complete ordered string count.
/// # Safety
/// Input is a live readable owner and output independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_strings_length(
    strings: *const ShiroRsStrings,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(strings, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe { result(output, || Ok((*strings).values.len())) }
}

/// Snapshot every UTF-8 byte of a checked string into independent ownership.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_strings_get(
    strings: *const ShiroRsStrings,
    index: usize,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(strings, 1) {
        return status;
    }
    // SAFETY: Live readable owner; the index is checked before snapshot creation.
    let Some(value) = (unsafe { &(*strings).values }).get(index) else {
        return 2;
    };
    // SAFETY: Immutable string and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: value.as_bytes().to_vec(),
            })))
        })
    }
}

/// Deep-copy all strings into ownership independent of the source lifetime.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_strings_clone(
    strings: *const ShiroRsStrings,
    output: *mut *mut ShiroRsStrings,
) -> u32 {
    if let Err(status) = range(strings, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsStrings {
                values: (*strings).values.clone(),
            })))
        })
    }
}

/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_strings_release(slot: *mut *mut ShiroRsStrings) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}

/// Deep-copy complete native entries in order without restricting public fields.
/// # Safety
/// Input is readable aligned storage for count descriptors containing live
/// immutable owners; empty input permits null. Output is independent aligned
/// writable storage holding no live owner on success. Failed output is unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_index_entries_create(
    entries: *const ShiroRsIndexEntryInput,
    count: usize,
    output: *mut *mut ShiroRsIndexEntries,
) -> u32 {
    if let Err(status) = range(entries, count) {
        return status;
    }
    // SAFETY: Complete descriptor array is readable; nested pointers checked below.
    let entries = unsafe { input(entries, count) };
    for entry in entries {
        for status in [range(entry.stem, 1), range(entry.phonemes, 1)] {
            if let Err(status) = status {
                return status;
            }
        }
    }
    // SAFETY: All owners live and immutable; output independent.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            values.try_reserve_exact(count).map_err(io::Error::other)?;
            for entry in entries {
                values.push(crate::index::Entry {
                    stem: (*entry.stem).value.clone(),
                    phonemes: (*entry.phonemes).values.clone(),
                });
            }
            Ok(Box::into_raw(Box::new(ShiroRsIndexEntries { values })))
        })
    }
}

/// Retrieve the complete ordered entry count.
/// # Safety
/// Input is a live readable owner and output independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_index_entries_length(
    entries: *const ShiroRsIndexEntries,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(entries, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe { result(output, || Ok((*entries).values.len())) }
}

/// Snapshot every native path unit of a checked entry into independent ownership.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_index_entries_get_stem(
    entries: *const ShiroRsIndexEntries,
    index: usize,
    output: *mut *mut ShiroRsPath,
) -> u32 {
    if let Err(status) = range(entries, 1) {
        return status;
    }
    // SAFETY: Live readable owner; index checked before snapshot creation.
    let Some(entry) = (unsafe { &(*entries).values }).get(index) else {
        return 2;
    };
    // SAFETY: Immutable entry and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsPath {
                value: entry.stem.clone(),
            })))
        })
    }
}

/// Snapshot all phonemes of a checked entry in order into independent ownership.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_index_entries_get_phonemes(
    entries: *const ShiroRsIndexEntries,
    index: usize,
    output: *mut *mut ShiroRsStrings,
) -> u32 {
    if let Err(status) = range(entries, 1) {
        return status;
    }
    // SAFETY: Live readable owner; index checked before snapshot creation.
    let Some(entry) = (unsafe { &(*entries).values }).get(index) else {
        return 2;
    };
    // SAFETY: Immutable entry and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsStrings {
                values: entry.phonemes.clone(),
            })))
        })
    }
}

/// Deep-copy every complete entry into ownership independent of the source.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_index_entries_clone(
    entries: *const ShiroRsIndexEntries,
    output: *mut *mut ShiroRsIndexEntries,
) -> u32 {
    if let Err(status) = range(entries, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsIndexEntries {
                values: (*entries).values.clone(),
            })))
        })
    }
}

/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_index_entries_release(
    slot: *mut *mut ShiroRsIndexEntries,
) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}

/// Parse complete index bytes using native directory joins and phoneme padding.
/// Blank rows, CRLF, literal spaces and physical-line errors follow native read.
/// # Safety
/// Inputs are live readable owners; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_index_read_bytes(
    bytes: *const ShiroRsBytes,
    directory: *const ShiroRsPath,
    left: *const ShiroRsStrings,
    right: *const ShiroRsStrings,
    output: *mut *mut ShiroRsIndexEntries,
) -> u32 {
    for status in [
        range(bytes, 1),
        range(directory, 1),
        range(left, 1),
        range(right, 1),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable complete inputs and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsIndexEntries {
                values: crate::index::read(
                    Cursor::new(&(*bytes).values),
                    &(*directory).value,
                    &(*left).values,
                    &(*right).values,
                )?,
            })))
        })
    }
}
