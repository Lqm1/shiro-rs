//! Complete native batch options, extractor variants and extraction outputs.
use super::{
    ShiroRsAudioOptions, ShiroRsBytes, ShiroRsFeatureOptions, ShiroRsPath, ShiroRsUniformCallback,
    buffers::release, range, result,
};
use crate::batch::{self, Extractor, Options, Outputs, Preset, SptkPrograms};
use std::{ffi::c_void, io};

/// Independent complete native batch settings and input suffix.
pub struct ShiroRsBatchOptions {
    value: Options,
}
/// Independent complete Native, SPTK or Lua extractor.
pub struct ShiroRsExtractor {
    value: Extractor,
}
/// Independent complete raw, parameter and optional MFCC output paths.
pub struct ShiroRsBatchOutputs {
    value: Outputs,
}

fn preset(code: u32) -> Result<Preset, u32> {
    match code {
        0 => Ok(Preset::Mfcc12Da16k),
        1 => Ok(Preset::Mfcc12Dae16k),
        2 => Ok(Preset::Plpcc12Da16k),
        _ => Err(2),
    }
}
fn preset_code(value: Preset) -> u32 {
    match value {
        Preset::Mfcc12Da16k => 0,
        Preset::Mfcc12Dae16k => 1,
        Preset::Plpcc12Da16k => 2,
    }
}

/// All native settings for preset zero MFCC-DA, one MFCC-DAE, two PLPCC-DA.
/// # Safety
/// Output is independent aligned writable descriptor storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_preset_feature_options(
    code: u32,
    output: *mut ShiroRsFeatureOptions,
) -> u32 {
    let value = match preset(code) {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: Independent writable output.
    unsafe { result(output, || Ok(value.feature_options().into())) }
}
/// Construct complete native default batch options.
/// # Safety
/// Output is independent aligned writable storage holding no live owner on success.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_options_default(
    output: *mut *mut ShiroRsBatchOptions,
) -> u32 {
    // SAFETY: Independent writable output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBatchOptions {
                value: Options::default(),
            })))
        })
    }
}
/// Construct every batch setting, retaining full UTF-8 including empty/NUL suffixes.
/// Numerical audio validation remains native extraction's responsibility.
/// # Safety
/// Audio is aligned initialized readable metadata; suffix is a live readable byte
/// owner. Output is independent aligned writable storage holding no live owner on
/// success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_options_create(
    audio: *const ShiroRsAudioOptions,
    extension: *const ShiroRsBytes,
    output: *mut *mut ShiroRsBatchOptions,
) -> u32 {
    for status in [range(audio, 1), range(extension, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Complete initialized readable audio metadata.
    let audio = match unsafe { audio.read() }.native() {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: Live immutable suffix and independent output.
    unsafe {
        result(output, || {
            let input_extension = std::str::from_utf8(&(*extension).values)
                .map_err(io::Error::other)?
                .to_owned();
            Ok(Box::into_raw(Box::new(ShiroRsBatchOptions {
                value: Options {
                    audio,
                    input_extension,
                },
            })))
        })
    }
}
/// Copy every audio setting, including optional-rate presence.
/// # Safety
/// Input is a live readable owner and output independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_options_get_audio(
    options: *const ShiroRsBatchOptions,
    output: *mut ShiroRsAudioOptions,
) -> u32 {
    if let Err(status) = range(options, 1) {
        return status;
    }
    // SAFETY: Live readable input and independent output.
    unsafe { result(output, || Ok((*options).value.audio.into())) }
}
/// Snapshot every suffix byte into independent ownership.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_options_get_input_extension(
    options: *const ShiroRsBatchOptions,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(options, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBytes {
                values: (*options).value.input_extension.as_bytes().to_vec(),
            })))
        })
    }
}
/// Deep-copy every native batch option into independent ownership.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_options_clone(
    options: *const ShiroRsBatchOptions,
    output: *mut *mut ShiroRsBatchOptions,
) -> u32 {
    if let Err(status) = range(options, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBatchOptions {
                value: (*options).value.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_options_release(
    slot: *mut *mut ShiroRsBatchOptions,
) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}

/// Construct a native extractor with the complete selected preset.
/// # Safety
/// Output is independent aligned writable storage holding no live owner on success.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_native(
    code: u32,
    output: *mut *mut ShiroRsExtractor,
) -> u32 {
    let value = match preset(code) {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: Independent writable output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsExtractor {
                value: Extractor::Native(value),
            })))
        })
    }
}
/// Construct SPTK with every default program path from the native implementation.
/// # Safety
/// Output is independent aligned writable storage holding no live owner on success.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_sptk_default(
    output: *mut *mut ShiroRsExtractor,
) -> u32 {
    // SAFETY: Independent writable output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsExtractor {
                value: Extractor::Sptk(SptkPrograms::default()),
            })))
        })
    }
}
/// Construct all three SPTK program paths without Unicode conversion.
/// # Safety
/// All paths are live readable owners, including repeated inputs. Output is
/// independent aligned writable storage holding no live owner on success.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_sptk(
    frame: *const ShiroRsPath,
    mfcc: *const ShiroRsPath,
    delta: *const ShiroRsPath,
    output: *mut *mut ShiroRsExtractor,
) -> u32 {
    for status in [range(frame, 1), range(mfcc, 1), range(delta, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable paths and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsExtractor {
                value: Extractor::Sptk(SptkPrograms {
                    frame: (*frame).value.clone(),
                    mfcc: (*mfcc).value.clone(),
                    delta: (*delta).value.clone(),
                }),
            })))
        })
    }
}
/// Construct the complete Lua interpreter, script and executable directory.
/// # Safety
/// All paths are live readable owners, including repeated inputs. Output is
/// independent aligned writable storage holding no live owner on success.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_lua(
    interpreter: *const ShiroRsPath,
    script: *const ShiroRsPath,
    executable_directory: *const ShiroRsPath,
    output: *mut *mut ShiroRsExtractor,
) -> u32 {
    for status in [
        range(interpreter, 1),
        range(script, 1),
        range(executable_directory, 1),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable paths and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsExtractor {
                value: Extractor::Lua {
                    interpreter: (*interpreter).value.clone(),
                    script: (*script).value.clone(),
                    executable_directory: (*executable_directory).value.clone(),
                },
            })))
        })
    }
}
/// Retrieve variant zero native, one SPTK, two Lua.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_kind(
    extractor: *const ShiroRsExtractor,
    output: *mut u32,
) -> u32 {
    if let Err(status) = range(extractor, 1) {
        return status;
    }
    // SAFETY: Live readable input and independent output.
    unsafe {
        result(output, || {
            Ok(match &(*extractor).value {
                Extractor::Native(_) => 0,
                Extractor::Sptk(_) => 1,
                Extractor::Lua { .. } => 2,
            })
        })
    }
}
/// Retrieve a native preset code; other extractor variants return range status.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_get_preset(
    extractor: *const ShiroRsExtractor,
    output: *mut u32,
) -> u32 {
    if let Err(status) = range(extractor, 1) {
        return status;
    }
    // SAFETY: Live readable input.
    let Extractor::Native(value) = (unsafe { &(*extractor).value }) else {
        return 2;
    };
    // SAFETY: Immutable preset and independent output.
    unsafe { result(output, || Ok(preset_code(*value))) }
}
/// Snapshot path zero/one/two: SPTK frame/mfcc/delta or Lua interpreter/script/
/// executable directory. Native extractors and invalid indices return range status.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_get_path(
    extractor: *const ShiroRsExtractor,
    index: usize,
    output: *mut *mut ShiroRsPath,
) -> u32 {
    if let Err(status) = range(extractor, 1) {
        return status;
    }
    // SAFETY: Live readable input and checked variant/index.
    let paths = match unsafe { &(*extractor).value } {
        Extractor::Native(_) => return 2,
        Extractor::Sptk(value) => [&value.frame, &value.mfcc, &value.delta],
        Extractor::Lua {
            interpreter,
            script,
            executable_directory,
        } => [interpreter, script, executable_directory],
    };
    let Some(value) = paths.get(index) else {
        return 2;
    };
    // SAFETY: Immutable path and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsPath {
                value: (*value).clone(),
            })))
        })
    }
}
/// Deep-copy every variant field into independent ownership.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_clone(
    extractor: *const ShiroRsExtractor,
    output: *mut *mut ShiroRsExtractor,
) -> u32 {
    if let Err(status) = range(extractor, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsExtractor {
                value: (*extractor).value.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_extractor_release(slot: *mut *mut ShiroRsExtractor) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}

/// Construct arbitrary complete Outputs fields without imposing filesystem rules.
/// A null MFCC owner is absent; a live empty path is a present empty path.
/// # Safety
/// Required paths and optional nonnull MFCC are live readable owners. Output is
/// independent aligned writable storage holding no live owner on success.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_outputs_create(
    raw: *const ShiroRsPath,
    parameters: *const ShiroRsPath,
    mfcc: *const ShiroRsPath,
    output: *mut *mut ShiroRsBatchOutputs,
) -> u32 {
    for status in [
        range(raw, 1),
        range(parameters, 1),
        range(mfcc, usize::from(!mfcc.is_null())),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable required/optional paths and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBatchOutputs {
                value: Outputs {
                    raw: (*raw).value.clone(),
                    parameters: (*parameters).value.clone(),
                    mfcc: if mfcc.is_null() {
                        None
                    } else {
                        Some((*mfcc).value.clone())
                    },
                },
            })))
        })
    }
}
/// Retrieve exact optional MFCC presence as zero or one.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_outputs_has_mfcc(
    outputs: *const ShiroRsBatchOutputs,
    output: *mut u32,
) -> u32 {
    if let Err(status) = range(outputs, 1) {
        return status;
    }
    // SAFETY: Live readable input and independent output.
    unsafe { result(output, || Ok(u32::from((*outputs).value.mfcc.is_some()))) }
}
/// Snapshot path zero raw, one parameters or two present MFCC. An absent MFCC
/// or invalid index returns range status with output unchanged.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_outputs_get_path(
    outputs: *const ShiroRsBatchOutputs,
    index: usize,
    output: *mut *mut ShiroRsPath,
) -> u32 {
    if let Err(status) = range(outputs, 1) {
        return status;
    }
    // SAFETY: Live readable input and checked field index.
    let value = unsafe { &(*outputs).value };
    let path = match index {
        0 => Some(&value.raw),
        1 => Some(&value.parameters),
        2 => value.mfcc.as_ref(),
        _ => None,
    };
    let Some(path) = path else {
        return 2;
    };
    // SAFETY: Immutable path and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsPath {
                value: path.clone(),
            })))
        })
    }
}
/// Deep-copy every output path and optional presence into independent ownership.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_outputs_clone(
    outputs: *const ShiroRsBatchOutputs,
    output: *mut *mut ShiroRsBatchOutputs,
) -> u32 {
    if let Err(status) = range(outputs, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsBatchOutputs {
                value: (*outputs).value.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_outputs_release(
    slot: *mut *mut ShiroRsBatchOutputs,
) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}

/// Delegate the complete native file extraction workflow with every option/variant.
/// Filesystem and RNG effects already performed are not rolled back after failure.
/// # Safety
/// Inputs are live immutable owners. Callback/context satisfy UniformCallback's
/// synchronous non-unwinding independent-context contract. Output is independent
/// aligned writable storage holding no live owner on success. Failed output
/// remains unchanged and is validated before filesystem or callback effects.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_batch_extract_file(
    stem: *const ShiroRsPath,
    options: *const ShiroRsBatchOptions,
    extractor: *const ShiroRsExtractor,
    callback: ShiroRsUniformCallback,
    context: *mut c_void,
    output: *mut *mut ShiroRsBatchOutputs,
) -> u32 {
    for status in [range(stem, 1), range(options, 1), range(extractor, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable owners, independent output and synchronous callback.
    unsafe {
        result(output, || {
            let value = batch::extract_file(
                &(*stem).value,
                &(*options).value,
                &(*extractor).value,
                || {
                    let mut draw = f32::NAN;
                    match callback {
                        Some(callback) if callback(context, &mut draw) == 0 => draw,
                        _ => f32::NAN,
                    }
                },
            )
            .map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsBatchOutputs { value })))
        })
    }
}
