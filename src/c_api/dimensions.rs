//! Lossless target-width arrays and standalone complete model dimensions.
use super::{
    ShiroRsModel,
    buffers::{input, release},
    range, result,
};
use std::io;

/// Independently owned target-width unsigned integers.
pub struct ShiroRsArrayUsize {
    values: Vec<usize>,
}

fn copied(values: &[usize]) -> io::Result<Vec<usize>> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(values.len())
        .map_err(io::Error::other)?;
    output.extend_from_slice(values);
    Ok(output)
}

/// Copy every integer without narrowing or restricting its value.
/// # Safety
/// Input is aligned initialized readable storage for the stated count; empty
/// input permits null. Output is independent aligned writable storage holding
/// no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_usize_create(
    values: *const usize,
    count: usize,
    output: *mut *mut ShiroRsArrayUsize,
) -> u32 {
    if let Err(status) = range(values, count) {
        return status;
    }
    // SAFETY: Valid readable input and independent writable output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayUsize {
                values: copied(input(values, count))?,
            })))
        })
    }
}

/// Retrieve the complete ordered value count.
/// # Safety
/// Input is a live readable owner and output independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_usize_length(
    values: *const ShiroRsArrayUsize,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(values, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe { result(output, || Ok((*values).values.len())) }
}

/// Copy a complete checked range, retaining the output buffer on failure.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage for the stated count and must not alias input. Empty output permits
/// null. Neither participating owner nor storage may be mutated concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_usize_copy(
    values: *const ShiroRsArrayUsize,
    offset: usize,
    output: *mut usize,
    count: usize,
) -> u32 {
    if let Err(status) = range(values, 1) {
        return status;
    }
    if let Err(status) = range(output.cast_const(), count) {
        return status;
    }
    let Some(end) = offset.checked_add(count) else {
        return 2;
    };
    // SAFETY: Input is a live readable owner.
    let Some(source) = (unsafe { &(*values).values }).get(offset..end) else {
        return 2;
    };
    if count != 0 {
        // SAFETY: Both complete ranges validated; output does not overlap input.
        unsafe { std::ptr::copy_nonoverlapping(source.as_ptr(), output, count) };
    }
    0
}

/// Deep-copy all integers into an owner independent of the source lifetime.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_usize_clone(
    values: *const ShiroRsArrayUsize,
    output: *mut *mut ShiroRsArrayUsize,
) -> u32 {
    if let Err(status) = range(values, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayUsize {
                values: copied(&(*values).values)?,
            })))
        })
    }
}

/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner
/// or null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_usize_release(slot: *mut *mut ShiroRsArrayUsize) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}

/// Retrieve every model stream dimension in original order with native validation.
/// # Safety
/// Model is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_model_dimensions(
    model: *const ShiroRsModel,
    output: *mut *mut ShiroRsArrayUsize,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    // SAFETY: Live immutable model and independent output storage.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayUsize {
                values: crate::dataset::dimensions(&(*model).value)?,
            })))
        })
    }
}
