//! Full native alignment settings and both memory and host-document workflows.
use super::{ShiroRsBytes, ShiroRsModel, ShiroRsObservation, ShiroRsStates, range, result};
use crate::{
    alignment::{self, DurationMode, Options},
    labels::SegmentationDocument,
};
use liblrhsmm_rs::{GeometricOptions, HsmmOptions};
use std::io;

/// All native alignment settings. Integer flags must be 0 or 1. Layout follows
/// the target C ABI; duration_extra has the target size_t width.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiroRsAlignmentOptions {
    /// 0 explicit HSMM durations, 1 geometric HMM durations.
    pub duration_mode: u32,
    /// 0 embedded alignment, 1 isolated phone groups.
    pub isolated: u32,
    pub hsmm_temperature: f32,
    pub duration_weight: f32,
    pub state_radius: f32,
    pub duration_extra: usize,
    pub duration_extra_factor: f32,
    pub geometric_temperature: f32,
    pub pruning_slope: f32,
}

impl From<Options> for ShiroRsAlignmentOptions {
    fn from(options: Options) -> Self {
        Self {
            duration_mode: match options.duration_mode {
                DurationMode::Explicit => 0,
                DurationMode::Geometric => 1,
            },
            isolated: u32::from(options.isolated),
            hsmm_temperature: options.hsmm.temperature,
            duration_weight: options.hsmm.duration_weight,
            state_radius: options.hsmm.state_radius,
            duration_extra: options.hsmm.duration_extra,
            duration_extra_factor: options.hsmm.duration_extra_factor,
            geometric_temperature: options.geometric.temperature,
            pruning_slope: options.geometric.pruning_slope,
        }
    }
}

impl TryFrom<ShiroRsAlignmentOptions> for Options {
    type Error = u32;
    fn try_from(options: ShiroRsAlignmentOptions) -> Result<Self, Self::Error> {
        let duration_mode = match options.duration_mode {
            0 => DurationMode::Explicit,
            1 => DurationMode::Geometric,
            _ => return Err(2),
        };
        let isolated = match options.isolated {
            0 => false,
            1 => true,
            _ => return Err(2),
        };
        Ok(Self {
            duration_mode,
            isolated,
            hsmm: HsmmOptions {
                temperature: options.hsmm_temperature,
                duration_weight: options.duration_weight,
                state_radius: options.state_radius,
                duration_extra: options.duration_extra,
                duration_extra_factor: options.duration_extra_factor,
            },
            geometric: GeometricOptions {
                temperature: options.geometric_temperature,
                pruning_slope: options.pruning_slope,
            },
        })
    }
}

/// Copy the unchanged native defaults into caller-provided descriptor storage.
/// # Safety
/// Output is independent aligned exclusively writable storage for one descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_alignment_options_default(
    output: *mut ShiroRsAlignmentOptions,
) -> u32 {
    // SAFETY: Independent validated descriptor output under the public contract.
    unsafe { result(output, || Ok(Options::default().into())) }
}

/// Align all input states without modifying any participating input owner.
/// # Safety
/// Inputs are live readable owners from this library. Options are aligned live
/// initialized readable descriptor storage. Output is independent aligned writable
/// storage holding no live owner and remains unchanged on failure. No participating
/// owner or descriptor may be modified or released during this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_align_states(
    model: *const ShiroRsModel,
    observation: *const ShiroRsObservation,
    states: *const ShiroRsStates,
    options: *const ShiroRsAlignmentOptions,
    output: *mut *mut ShiroRsStates,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    if let Err(status) = range(observation, 1) {
        return status;
    }
    if let Err(status) = range(states, 1) {
        return status;
    }
    if let Err(status) = range(options, 1) {
        return status;
    }
    // SAFETY: Caller supplies live initialized readable descriptor storage.
    let options = match Options::try_from(unsafe { options.read() }) {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: All owners are readable and output storage is independent.
    unsafe {
        result(output, || {
            let value = alignment::align_states(
                &(*model).value,
                &(*observation).value,
                &(*states).value,
                options,
            )?;
            Ok(Box::into_raw(Box::new(ShiroRsStates { value })))
        })
    }
}

/// Align a complete original JSON document using the unchanged native file loader.
/// Filenames retain their process-relative meaning. All document, file and state
/// metadata is retained. Output is complete UTF-8 JSON in independent storage.
/// # Safety
/// Inputs are live readable owners from this library. Options are aligned live
/// initialized readable descriptor storage. Output is independent aligned writable
/// storage holding no live owner and remains unchanged on failure. No participating
/// owner or descriptor may be modified or released during this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_align_document(
    model: *const ShiroRsModel,
    document: *const ShiroRsBytes,
    options: *const ShiroRsAlignmentOptions,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    if let Err(status) = range(document, 1) {
        return status;
    }
    if let Err(status) = range(options, 1) {
        return status;
    }
    // SAFETY: Caller supplies live initialized readable descriptor storage.
    let options = match Options::try_from(unsafe { options.read() }) {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: All owners are readable and output storage is independent.
    unsafe {
        result(output, || {
            let document: SegmentationDocument =
                serde_json::from_slice(&(*document).values).map_err(io::Error::other)?;
            let aligned = alignment::align_document(&(*model).value, &document, options)?;
            let values = serde_json::to_vec(&aligned).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}
