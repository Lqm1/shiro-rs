//! Independent host paths without lossy Unicode conversions.
use super::{ShiroRsBytes, buffers::release, range, result};
use std::{io, path::PathBuf};

/// Complete independent native path, including non-Unicode platform strings.
pub struct ShiroRsPath {
    pub(super) value: PathBuf,
}

#[cfg(windows)]
fn decode(bytes: &[u8]) -> io::Result<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    if !bytes.len().is_multiple_of(2) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "native Windows path requires complete u16 units",
        ));
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    Ok(std::ffi::OsString::from_wide(&units).into())
}
#[cfg(unix)]
fn decode(bytes: &[u8]) -> io::Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    Ok(std::ffi::OsString::from_vec(bytes.to_vec()).into())
}
#[cfg(not(any(windows, unix)))]
fn decode(_: &[u8]) -> io::Result<PathBuf> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "native path encoding is unavailable on this target",
    ))
}
#[cfg(windows)]
fn encode(path: &std::path::Path) -> io::Result<Vec<u8>> {
    use std::os::windows::ffi::OsStrExt;
    Ok(path
        .as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect())
}
#[cfg(unix)]
fn encode(path: &std::path::Path) -> io::Result<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    Ok(path.as_os_str().as_bytes().to_vec())
}
#[cfg(not(any(windows, unix)))]
fn encode(_: &std::path::Path) -> io::Result<Vec<u8>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "native path encoding is unavailable on this target",
    ))
}

/// Native wire encoding: one Unix bytes, two Windows little-endian u16 units,
/// zero unavailable. Explicit lengths include embedded NUL; no terminator is added.
#[unsafe(no_mangle)]
pub extern "C" fn shiro_rs_path_native_encoding() -> u32 {
    if cfg!(windows) {
        2
    } else if cfg!(unix) {
        1
    } else {
        0
    }
}
/// Construct a complete native path from platform bytes without Unicode loss.
/// # Safety
/// Input is a live readable byte owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_path_from_native_bytes(
    bytes: *const ShiroRsBytes,
    output: *mut *mut ShiroRsPath,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsPath {
                value: decode(&(*bytes).values)?,
            })))
        })
    }
}
/// Construct from a complete UTF-8 string, including embedded NUL characters.
/// # Safety
/// Input is a live readable byte owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_path_from_utf8(
    bytes: *const ShiroRsBytes,
    output: *mut *mut ShiroRsPath,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            let value = std::str::from_utf8(&(*bytes).values).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsPath {
                value: PathBuf::from(value),
            })))
        })
    }
}
/// Snapshot all platform-native units into independent bytes without a terminator.
/// # Safety
/// Input is a live readable path owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_path_native_bytes(
    path: *const ShiroRsPath,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(path, 1) {
        return status;
    }
    // SAFETY: Live immutable path and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: encode(&(*path).value)?,
            })))
        })
    }
}
/// Append an entire UTF-8 suffix through the native index operation.
/// # Safety
/// Inputs are live readable owners; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_path_append_suffix(
    path: *const ShiroRsPath,
    suffix: *const ShiroRsBytes,
    output: *mut *mut ShiroRsPath,
) -> u32 {
    for status in [range(path, 1), range(suffix, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable inputs and independent output.
    unsafe {
        result(output, || {
            let suffix = std::str::from_utf8(&(*suffix).values).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsPath {
                value: crate::index::append_suffix(&(*path).value, suffix),
            })))
        })
    }
}
/// Deep-copy every native path unit into independent ownership.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_path_clone(
    path: *const ShiroRsPath,
    output: *mut *mut ShiroRsPath,
) -> u32 {
    if let Err(status) = range(path, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsPath {
                value: (*path).value.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_path_release(slot: *mut *mut ShiroRsPath) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}
