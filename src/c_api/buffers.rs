//! Independent bytes and binary32 arrays; no borrowed internal pointers escape.
use super::{boundary, range, result};
use std::{io, panic::RefUnwindSafe};

unsafe fn input<'a, T>(pointer: *const T, count: usize) -> &'a [T] {
    if count == 0 {
        &[]
    } else {
        // SAFETY: Public boundary checked the range; caller guarantees readable validity.
        unsafe { std::slice::from_raw_parts(pointer, count) }
    }
}

fn copied<T: Copy>(source: &[T]) -> io::Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(source.len())
        .map_err(io::Error::other)?;
    values.extend_from_slice(source);
    Ok(values)
}

pub(super) unsafe fn release<T: RefUnwindSafe>(slot: *mut *mut T) -> u32 {
    if let Err(status) = range(slot.cast_const(), 1) {
        return status;
    }
    // SAFETY: Caller supplies initialized exclusively writable unique owner storage.
    let pointer = unsafe { slot.read() };
    if pointer.is_null() {
        return 0;
    }
    if let Err(status) = range(pointer, 1) {
        return status;
    }
    match boundary(|| {
        // SAFETY: Unique live owner; clear its independent slot before dropping.
        unsafe {
            slot.write(std::ptr::null_mut());
            drop(Box::from_raw(pointer));
        }
        Ok(())
    }) {
        Ok(()) => 0,
        Err(status) => status,
    }
}

/// Independent owned u8 values; retrieve data through checked copies.
pub struct ShiroRsBytes {
    pub(super) values: Vec<u8>,
}

/// Copy caller values into an independent owner, retaining all scalar bits.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_bytes_create(
    data: *const u8,
    count: usize,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(data, count) {
        return status;
    }
    // SAFETY: Readable borrowed input and independent validated output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: copied(input(data, count))?,
            })))
        })
    }
}

/// Copy the value count; errors retain the initialized output.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_bytes_length(
    handle: *const ShiroRsBytes,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(handle, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe { result(output, || Ok((*handle).values.len())) }
}

/// Copy a checked contiguous range; errors leave caller buffers unchanged.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_bytes_copy(
    handle: *const ShiroRsBytes,
    offset: usize,
    output: *mut u8,
    count: usize,
) -> u32 {
    if let Err(status) = range(handle, 1) {
        return status;
    }
    if let Err(status) = range(output.cast_const(), count) {
        return status;
    }
    let Some(end) = offset.checked_add(count) else {
        return 2;
    };
    // SAFETY: Live readable owner; caller's writable output is independent.
    let Some(values) = (unsafe { &(*handle).values }).get(offset..end) else {
        return 2;
    };
    if count != 0 {
        // SAFETY: Both checked ranges have count elements and do not overlap.
        unsafe {
            std::ptr::copy_nonoverlapping(values.as_ptr(), output, count);
        }
    }
    0
}

/// Deep-copy all values; the copy outlives the source owner.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_bytes_clone(
    handle: *const ShiroRsBytes,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(handle, 1) {
        return status;
    }
    // SAFETY: Live readable source and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: copied(&(*handle).values)?,
            })))
        })
    }
}

/// Release unique ownership and clear its initialized slot. A null owned value is allowed.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_bytes_release(owner: *mut *mut ShiroRsBytes) -> u32 {
    // SAFETY: Caller transfers unique ownership through an independent writable slot.
    unsafe { release(owner) }
}

/// Independent owned f32 values; retrieve data through checked copies.
pub struct ShiroRsArrayF32 {
    pub(super) values: Vec<f32>,
}

/// Copy caller values into an independent owner, retaining all scalar bits.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_f32_create(
    data: *const f32,
    count: usize,
    output: *mut *mut ShiroRsArrayF32,
) -> u32 {
    if let Err(status) = range(data, count) {
        return status;
    }
    // SAFETY: Readable borrowed input and independent validated output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayF32 {
                values: copied(input(data, count))?,
            })))
        })
    }
}

/// Copy the value count; errors retain the initialized output.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_f32_length(
    handle: *const ShiroRsArrayF32,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(handle, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe { result(output, || Ok((*handle).values.len())) }
}

/// Copy a checked contiguous range; errors leave caller buffers unchanged.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_f32_copy(
    handle: *const ShiroRsArrayF32,
    offset: usize,
    output: *mut f32,
    count: usize,
) -> u32 {
    if let Err(status) = range(handle, 1) {
        return status;
    }
    if let Err(status) = range(output.cast_const(), count) {
        return status;
    }
    let Some(end) = offset.checked_add(count) else {
        return 2;
    };
    // SAFETY: Live readable owner; caller's writable output is independent.
    let Some(values) = (unsafe { &(*handle).values }).get(offset..end) else {
        return 2;
    };
    if count != 0 {
        // SAFETY: Both checked ranges have count elements and do not overlap.
        unsafe {
            std::ptr::copy_nonoverlapping(values.as_ptr(), output, count);
        }
    }
    0
}

/// Deep-copy all values; the copy outlives the source owner.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_f32_clone(
    handle: *const ShiroRsArrayF32,
    output: *mut *mut ShiroRsArrayF32,
) -> u32 {
    if let Err(status) = range(handle, 1) {
        return status;
    }
    // SAFETY: Live readable source and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayF32 {
                values: copied(&(*handle).values)?,
            })))
        })
    }
}

/// Release unique ownership and clear its initialized slot. A null owned value is allowed.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_array_f32_release(owner: *mut *mut ShiroRsArrayF32) -> u32 {
    // SAFETY: Caller transfers unique ownership through an independent writable slot.
    unsafe { release(owner) }
}

/// Decode the original headerless little-endian binary32 stream with a scalar budget.
/// Partial final scalars and exceeded budgets retain initialized output slots.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_rawfloat_read_bytes(
    data: *const u8,
    count: usize,
    maximum_samples: usize,
    output: *mut *mut ShiroRsArrayF32,
) -> u32 {
    if let Err(status) = range(data, count) {
        return status;
    }
    // SAFETY: Readable borrowed bytes and independent output; native decoding owns its values.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayF32 {
                values: crate::rawfloat::read(input(data, count), maximum_samples)?,
            })))
        })
    }
}

/// Encode all binary32 bits into independent original-format bytes.
/// # Safety
/// Borrowed buffers are aligned initialized storage valid for their stated
/// lengths. Owners have the declared type and remain live for the call. Outputs
/// are independent aligned writable storage; constructors use slots holding no
/// live owner. No buffer aliases participating owners. Mutation and release
/// require exclusive access; release transfers unique ownership and clears the
/// initialized slot. Empty buffers permit null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_rawfloat_write_bytes(
    handle: *const ShiroRsArrayF32,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(handle, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output; native writer owns no borrowed buffers.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            crate::rawfloat::write(&mut values, &(*handle).values)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}
