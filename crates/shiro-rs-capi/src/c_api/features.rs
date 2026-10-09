//! Complete feature extraction settings and independent feature matrices.
use super::{ShiroRsArrayF32, buffers::release, range, result};
use crate::features::{self, Energy, FeatureKind, FeatureOptions, Features};
use std::io;

/// Every native feature option; integer codes are stable across targets.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShiroRsFeatureOptions {
    /// Zero MFCC, one MFBE, two PLPCC.
    pub kind: u32,
    pub order: usize,
    pub channels: usize,
    pub frame_length: usize,
    pub hop: f32,
    pub sample_rate_hz: f32,
    pub minimum_bandwidth_hz: f32,
    pub warp: f32,
    /// Flags accept only zero or one.
    pub include_dc: u32,
    /// Zero absent, one RMS, two decibels.
    pub energy: u32,
    pub delta: u32,
    pub acceleration: u32,
}
impl From<FeatureOptions> for ShiroRsFeatureOptions {
    fn from(options: FeatureOptions) -> Self {
        Self {
            kind: match options.kind {
                FeatureKind::Mfcc => 0,
                FeatureKind::Mfbe => 1,
                FeatureKind::Plpcc => 2,
            },
            order: options.order,
            channels: options.channels,
            frame_length: options.frame_length,
            hop: options.hop,
            sample_rate_hz: options.sample_rate_hz,
            minimum_bandwidth_hz: options.minimum_bandwidth_hz,
            warp: options.warp,
            include_dc: u32::from(options.include_dc),
            energy: match options.energy {
                None => 0,
                Some(Energy::Rms) => 1,
                Some(Energy::Decibels) => 2,
            },
            delta: u32::from(options.delta),
            acceleration: u32::from(options.acceleration),
        }
    }
}
impl ShiroRsFeatureOptions {
    fn native(self) -> Result<FeatureOptions, u32> {
        let flag = |value| -> Result<bool, u32> {
            match value {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(2),
            }
        };
        Ok(FeatureOptions {
            kind: match self.kind {
                0 => FeatureKind::Mfcc,
                1 => FeatureKind::Mfbe,
                2 => FeatureKind::Plpcc,
                _ => return Err(2),
            },
            order: self.order,
            channels: self.channels,
            frame_length: self.frame_length,
            hop: self.hop,
            sample_rate_hz: self.sample_rate_hz,
            minimum_bandwidth_hz: self.minimum_bandwidth_hz,
            warp: self.warp,
            include_dc: flag(self.include_dc)?,
            energy: match self.energy {
                0 => None,
                1 => Some(Energy::Rms),
                2 => Some(Energy::Decibels),
                _ => return Err(2),
            },
            delta: flag(self.delta)?,
            acceleration: flag(self.acceleration)?,
        })
    }
}
/// All feature matrix fields in independent ownership.
pub struct ShiroRsFeatures {
    pub(super) value: Features,
}
/// Complete native matrix dimensions; values are retrieved separately.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShiroRsFeatureInfo {
    pub frames: usize,
    pub columns: usize,
}

/// Copy every native feature default into the descriptor.
/// # Safety
/// Output is independent aligned exclusively writable descriptor storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_feature_options_default(
    output: *mut ShiroRsFeatureOptions,
) -> u32 {
    // SAFETY: Independent writable output under caller contract.
    unsafe { result(output, || Ok(FeatureOptions::default().into())) }
}

/// Construct all public matrix fields without imposing downstream validation.
/// Arbitrary dimensions and every binary32 value are retained independently.
/// # Safety
/// Values is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_features_create(
    frames: usize,
    columns: usize,
    values: *const ShiroRsArrayF32,
    output: *mut *mut ShiroRsFeatures,
) -> u32 {
    if let Err(status) = range(values, 1) {
        return status;
    }
    // SAFETY: Live readable values and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsFeatures {
                value: Features {
                    frames,
                    columns,
                    values: (*values).values.clone(),
                },
            })))
        })
    }
}

/// Extract using every native option and the complete input signal.
/// # Safety
/// Signal is a live readable owner; options is aligned initialized readable
/// storage. Output is independent aligned writable storage holding no live
/// owner on success. Inputs and failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_features_extract(
    signal: *const ShiroRsArrayF32,
    options: *const ShiroRsFeatureOptions,
    output: *mut *mut ShiroRsFeatures,
) -> u32 {
    for status in [range(signal, 1), range(options, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Checked readable initialized descriptor.
    let options = match unsafe { options.read() }.native() {
        Ok(options) => options,
        Err(status) => return status,
    };
    // SAFETY: Live immutable signal and independent output.
    unsafe {
        result(output, || {
            let value = features::extract(&(*signal).values, options).map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsFeatures { value })))
        })
    }
}

/// Retrieve both native dimensions without narrowing.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_features_get_info(
    features: *const ShiroRsFeatures,
    output: *mut ShiroRsFeatureInfo,
) -> u32 {
    if let Err(status) = range(features, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe {
        result(output, || {
            Ok(ShiroRsFeatureInfo {
                frames: (*features).value.frames,
                columns: (*features).value.columns,
            })
        })
    }
}

/// Snapshot every binary32 value into independent ownership.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_features_get_values(
    features: *const ShiroRsFeatures,
    output: *mut *mut ShiroRsArrayF32,
) -> u32 {
    if let Err(status) = range(features, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayF32 {
                values: (*features).value.values.clone(),
            })))
        })
    }
}

/// Deep-copy all fields into an owner independent of the source lifetime.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_features_clone(
    features: *const ShiroRsFeatures,
    output: *mut *mut ShiroRsFeatures,
) -> u32 {
    if let Err(status) = range(features, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsFeatures {
                value: (*features).value.clone(),
            })))
        })
    }
}

/// Release unique ownership and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner
/// or null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_features_release(slot: *mut *mut ShiroRsFeatures) -> u32 {
    // SAFETY: Caller transfers unique ownership through an initialized independent slot.
    unsafe { release(slot) }
}
