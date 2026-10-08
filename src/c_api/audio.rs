//! Complete audio preparation and caller-controlled or legacy dither sources.
use super::{ShiroRsArrayF32, ShiroRsBytes, buffers::release, range, result};
use crate::audio::{self, Audio, AudioOptions, DitherSequence};
use ciglet_rs::{
    resampling::{BoundaryPolicy, KernelPolicy},
    wave::{self, Encoding, Wave},
};
use std::{
    ffi::c_void,
    io::{self, Cursor},
};

/// Every public native wave header field; samples are supplied separately.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ShiroRsWaveInfo {
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub channels: u16,
    /// Zero PCM, one float. These codes do not change already decoded samples.
    pub encoding: u32,
}
/// All five native options, including explicit optional-rate presence.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShiroRsAudioOptions {
    pub normalize: u32,
    pub dither_level: f32,
    pub has_output_sample_rate: u32,
    pub output_sample_rate: u32,
    /// Zero includes sample zero; one preserves the legacy omission.
    pub boundary: u32,
    /// Zero stable; one preserves the original approximate kernel.
    pub kernel: u32,
}
impl From<AudioOptions> for ShiroRsAudioOptions {
    fn from(options: AudioOptions) -> Self {
        Self {
            normalize: u32::from(options.normalize),
            dither_level: options.dither_level,
            has_output_sample_rate: u32::from(options.output_sample_rate.is_some()),
            output_sample_rate: options.output_sample_rate.unwrap_or(0),
            boundary: match options.boundary {
                BoundaryPolicy::IncludeFirst => 0,
                BoundaryPolicy::LegacySkipFirst => 1,
            },
            kernel: match options.kernel {
                KernelPolicy::Stable => 0,
                KernelPolicy::Legacy => 1,
            },
        }
    }
}
impl ShiroRsAudioOptions {
    pub(super) fn native(self) -> Result<AudioOptions, u32> {
        Ok(AudioOptions {
            normalize: match self.normalize {
                0 => false,
                1 => true,
                _ => return Err(2),
            },
            dither_level: self.dither_level,
            output_sample_rate: match self.has_output_sample_rate {
                0 => None,
                1 => Some(self.output_sample_rate),
                _ => return Err(2),
            },
            boundary: match self.boundary {
                0 => BoundaryPolicy::IncludeFirst,
                1 => BoundaryPolicy::LegacySkipFirst,
                _ => return Err(2),
            },
            kernel: match self.kernel {
                0 => KernelPolicy::Stable,
                1 => KernelPolicy::Legacy,
                _ => return Err(2),
            },
        })
    }
}
/// Independent complete native audio result.
pub struct ShiroRsAudio {
    pub(super) value: Audio,
}
/// Exclusively mutable native legacy random sequence.
pub struct ShiroRsDitherSequence {
    value: DitherSequence,
}
/// Synchronous uniform source: write a finite value in `[0,1]` and return zero.
/// Nonzero status fails preparation. Output is borrowed only during the call.
/// Callback/context must remain valid, never unwind or mutate/release active
/// inputs. Already consumed draws cannot be rolled back after a later error.
/// Null is allowed when native preparation needs no draw; otherwise it fails.
pub type ShiroRsUniformCallback = Option<unsafe extern "C" fn(*mut c_void, *mut f32) -> u32>;

unsafe fn prepared(
    wave: Wave<f32>,
    options: AudioOptions,
    callback: ShiroRsUniformCallback,
    context: *mut c_void,
) -> io::Result<Audio> {
    audio::prepare(wave, options, || {
        let mut draw = f32::NAN;
        match callback {
            // SAFETY: Caller guarantees a live synchronous non-unwinding callback
            // and context; the initialized draw is writable only during this call.
            Some(callback) if unsafe { callback(context, &mut draw) } == 0 => draw,
            _ => f32::NAN,
        }
    })
    .map_err(io::Error::other)
}

/// Copy every native audio default.
/// # Safety
/// Output is independent aligned exclusively writable descriptor storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_audio_options_default(output: *mut ShiroRsAudioOptions) -> u32 {
    // SAFETY: Independent writable descriptor.
    unsafe { result(output, || Ok(AudioOptions::default().into())) }
}
/// Construct arbitrary public Audio fields, retaining every sample bit.
/// # Safety
/// Samples is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_audio_create(
    sample_rate: u32,
    samples: *const ShiroRsArrayF32,
    output: *mut *mut ShiroRsAudio,
) -> u32 {
    if let Err(status) = range(samples, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsAudio {
                value: Audio {
                    sample_rate,
                    samples: (*samples).values.clone(),
                },
            })))
        })
    }
}
/// Prepare decoded samples using every header/option field and a uniform source.
/// # Safety
/// Samples is a live readable owner; header/options are aligned initialized
/// readable descriptors. Output is independent aligned writable storage holding
/// no live owner on success. Callback/context obey ShiroRsUniformCallback.
/// Inputs and failed outputs remain unchanged; RNG consumption is not rolled back.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_audio_prepare(
    samples: *const ShiroRsArrayF32,
    header: *const ShiroRsWaveInfo,
    options: *const ShiroRsAudioOptions,
    callback: ShiroRsUniformCallback,
    context: *mut c_void,
    output: *mut *mut ShiroRsAudio,
) -> u32 {
    for status in [range(samples, 1), range(header, 1), range(options, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Readable initialized descriptors.
    let header = unsafe { header.read() };
    let encoding = match header.encoding {
        0 => Encoding::Pcm,
        1 => Encoding::Float,
        _ => return 2,
    };
    // SAFETY: Readable initialized descriptor.
    let options = match unsafe { options.read() }.native() {
        Ok(options) => options,
        Err(status) => return status,
    };
    // SAFETY: Immutable live input and valid synchronous callback; independent output.
    unsafe {
        result(output, || {
            let wave = Wave {
                sample_rate: header.sample_rate,
                bits_per_sample: header.bits_per_sample,
                channels: header.channels,
                encoding,
                samples: (*samples).values.clone(),
            };
            let value = prepared(wave, options, callback, context)?;
            Ok(Box::into_raw(Box::new(ShiroRsAudio { value })))
        })
    }
}
/// Decode complete WAVE bytes using the native bounded decoder, then prepare.
/// # Safety
/// Bytes is a live readable owner; options is an aligned initialized readable
/// descriptor. Output is independent aligned writable storage holding no live
/// owner on success. Callback/context obey ShiroRsUniformCallback. Failed outputs
/// remain unchanged; already consumed draws are not rolled back.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_audio_prepare_wave_bytes(
    bytes: *const ShiroRsBytes,
    maximum_samples: usize,
    options: *const ShiroRsAudioOptions,
    callback: ShiroRsUniformCallback,
    context: *mut c_void,
    output: *mut *mut ShiroRsAudio,
) -> u32 {
    for status in [range(bytes, 1), range(options, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Readable initialized descriptor.
    let options = match unsafe { options.read() }.native() {
        Ok(options) => options,
        Err(status) => return status,
    };
    // SAFETY: Live immutable bytes, valid synchronous callback and independent output.
    unsafe {
        result(output, || {
            let wave = wave::read(&mut Cursor::new(&(*bytes).values), maximum_samples)?;
            let value = prepared(wave, options, callback, context)?;
            Ok(Box::into_raw(Box::new(ShiroRsAudio { value })))
        })
    }
}
/// Copy the complete sample-rate field.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_audio_sample_rate(
    audio: *const ShiroRsAudio,
    output: *mut u32,
) -> u32 {
    if let Err(status) = range(audio, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe { result(output, || Ok((*audio).value.sample_rate)) }
}
/// Snapshot every sample into independent ownership.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_audio_get_samples(
    audio: *const ShiroRsAudio,
    output: *mut *mut ShiroRsArrayF32,
) -> u32 {
    if let Err(status) = range(audio, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayF32 {
                values: (*audio).value.samples.clone(),
            })))
        })
    }
}
/// Deep-copy every public audio field.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_audio_clone(
    audio: *const ShiroRsAudio,
    output: *mut *mut ShiroRsAudio,
) -> u32 {
    if let Err(status) = range(audio, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsAudio {
                value: (*audio).value.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_audio_release(slot: *mut *mut ShiroRsAudio) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}
/// Create the original Windows seed-one sequence.
/// # Safety
/// Output is independent aligned writable storage holding no live owner on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dither_windows(output: *mut *mut ShiroRsDitherSequence) -> u32 {
    // SAFETY: Independent writable output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsDitherSequence {
                value: DitherSequence::windows(),
            })))
        })
    }
}
/// Create the original Linux GNU seed-one sequence, including warmup.
/// # Safety
/// Output is independent aligned writable storage holding no live owner on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dither_linux_gnu(output: *mut *mut ShiroRsDitherSequence) -> u32 {
    // SAFETY: Independent writable output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsDitherSequence {
                value: DitherSequence::linux_gnu(),
            })))
        })
    }
}
/// Consume one draw. Invalid output storage is rejected before state mutation.
/// # Safety
/// Sequence is a live exclusively accessible owner. Output is independent aligned
/// writable storage. No concurrent mutation or release is permitted.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dither_next_uniform(
    sequence: *mut ShiroRsDitherSequence,
    output: *mut f32,
) -> u32 {
    if let Err(status) = range(sequence, 1) {
        return status;
    }
    // SAFETY: Exclusive sequence, independent output; result validates output first.
    unsafe { result(output, || Ok((*sequence).value.next_uniform())) }
}
/// Release a unique sequence and clear its slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dither_release(slot: *mut *mut ShiroRsDitherSequence) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}
