//! SHIRO-owned complete models. Binary interchange avoids cross-library owners.
use super::{ShiroRsBytes, buffers::release, range, result};
use crate::definition::ModelDefinition;
use liblrhsmm_rs::{
    Model,
    serial::{DecodeLimits, ModelEncoding},
};
use std::io;

/// Independent complete native model, including every stream and duration field.
pub struct ShiroRsModel {
    pub(super) value: Model,
}

/// Build a model from the original UTF-8 JSON definition and its native defaults.
/// # Safety
/// Inputs are live owners created by this library. The output is independent,
/// aligned writable storage holding no live owner; it remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_model_from_definition(
    definition: *const ShiroRsBytes,
    output: *mut *mut ShiroRsModel,
) -> u32 {
    if let Err(status) = range(definition, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output under the public contract.
    unsafe {
        result(output, || {
            let definition: ModelDefinition =
                serde_json::from_slice(&(*definition).values).map_err(io::Error::other)?;
            let value = definition.build().map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsModel { value })))
        })
    }
}

/// Read a complete original model, reject trailing bytes and bound array entries.
/// Supports historical models with and without variance-floor fields.
/// # Safety
/// Input is a live owner created by this library. The output is independent,
/// aligned writable storage holding no live owner; it remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_model_read_bytes(
    bytes: *const ShiroRsBytes,
    maximum_array_entries: usize,
    output: *mut *mut ShiroRsModel,
) -> u32 {
    if let Err(status) = range(bytes, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output under the public contract.
    unsafe {
        result(output, || {
            let value = Model::read_with_limits(
                (*bytes).values.as_slice(),
                DecodeLimits {
                    max_array_entries: maximum_array_entries,
                },
            )?;
            Ok(Box::into_raw(Box::new(ShiroRsModel { value })))
        })
    }
}

/// Write a complete original model. Encoding 0 includes variance floors; 1 omits
/// them and rejects nonzero floors instead of losing parameters. Other codes fail.
/// # Safety
/// Input is a live readable owner. The output is independent aligned writable
/// storage holding no live owner; it remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_model_write_bytes(
    model: *const ShiroRsModel,
    encoding: u32,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    let encoding = match encoding {
        0 => ModelEncoding::WithVarianceFloors,
        1 => ModelEncoding::WithoutVarianceFloors,
        _ => return 2,
    };
    // SAFETY: Live readable owner and independent output under the public contract.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            (*model).value.write_with_encoding(&mut values, encoding)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Clone every model parameter into independent storage.
/// # Safety
/// Input is a live readable owner. The output is independent aligned writable
/// storage holding no live owner; it remains unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_model_clone(
    model: *const ShiroRsModel,
    output: *mut *mut ShiroRsModel,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output under the public contract.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsModel {
                value: (*model).value.clone(),
            })))
        })
    }
}

/// Release a unique model and clear its initialized slot. An empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage containing a unique live model
/// from this library or null. No other call may use the model during release.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_model_release(slot: *mut *mut ShiroRsModel) -> u32 {
    // SAFETY: The public contract transfers this unique allocation to release.
    unsafe { release(slot) }
}
