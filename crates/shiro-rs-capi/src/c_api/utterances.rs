//! Complete native utterance workflows and every intermediate result field.
use super::{
    ShiroRsArrayF32, ShiroRsAudio, ShiroRsBytes, ShiroRsFeatures, ShiroRsIterationReports,
    ShiroRsLabels, ShiroRsModel, ShiroRsModelDefinition, ShiroRsPhoneMap,
    ShiroRsSegmentationDocument, ShiroRsStrings, ShiroRsUniformCallback, ShiroRsWaveInfo,
    buffers::release, range, result,
};
use crate::{
    features::FeatureKind,
    utterances::{self, ModelSource, Options, SegmentedUtterances, SegmentedWave},
};
use shiro_rs::dsp::wave::{Encoding, Wave};
use std::{ffi::c_void, io};

/// Every native utterance option; numerical validation remains native.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiroRsUtteranceOptions {
    pub utterances: usize,
    pub hop_seconds: f64,
    pub minimum_silence_seconds: f64,
    pub minimum_voicing_seconds: f64,
    pub iterations: usize,
}
impl From<Options> for ShiroRsUtteranceOptions {
    fn from(value: Options) -> Self {
        Self {
            utterances: value.utterances,
            hop_seconds: value.hop_seconds,
            minimum_silence_seconds: value.minimum_silence_seconds,
            minimum_voicing_seconds: value.minimum_voicing_seconds,
            iterations: value.iterations,
        }
    }
}
impl From<ShiroRsUtteranceOptions> for Options {
    fn from(value: ShiroRsUtteranceOptions) -> Self {
        Self {
            utterances: value.utterances,
            hop_seconds: value.hop_seconds,
            minimum_silence_seconds: value.minimum_silence_seconds,
            minimum_voicing_seconds: value.minimum_voicing_seconds,
            iterations: value.iterations,
        }
    }
}
/// Mode zero fresh, one initialized, two trained. Fresh ignores model entirely;
/// other modes borrow a live immutable model for the synchronous workflow.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShiroRsModelSource {
    pub mode: u32,
    pub model: *const ShiroRsModel,
}
impl ShiroRsModelSource {
    fn validate(self) -> Result<(), u32> {
        match self.mode {
            0 => Ok(()),
            1 | 2 => range(self.model, 1),
            _ => Err(2),
        }
    }
    unsafe fn native<'a>(self) -> ModelSource<'a> {
        match self.mode {
            1 => {
                // SAFETY: Validated live immutable model under caller's contract.
                ModelSource::Initialized(unsafe { &(*self.model).value })
            }
            2 => {
                // SAFETY: Validated live immutable model under caller's contract.
                ModelSource::Trained(unsafe { &(*self.model).value })
            }
            _ => ModelSource::Fresh,
        }
    }
}
/// All ten native utterance result fields. Null optional model pointers are
/// absent; all other owners are required and copied without workflow validation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShiroRsUtterancesInput {
    pub phonemap: *const ShiroRsPhoneMap,
    pub definition: *const ShiroRsModelDefinition,
    pub phones: *const ShiroRsStrings,
    pub initial_segmentation: *const ShiroRsSegmentationDocument,
    pub uninitialized_model: *const ShiroRsModel,
    pub initialized_model: *const ShiroRsModel,
    pub model: *const ShiroRsModel,
    pub iterations: *const ShiroRsIterationReports,
    pub alignment: *const ShiroRsSegmentationDocument,
    pub labels: *const ShiroRsLabels,
}
/// Complete decoded wave and feature-selection inputs. Encoding codes zero PCM,
/// one float; feature kind zero MFCC, one MFBE, two PLPCC. All other fields retain
/// their native values, including headers unused by decoded-sample preparation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShiroRsWaveSplitInput {
    pub header: ShiroRsWaveInfo,
    pub samples: *const ShiroRsArrayF32,
    pub filename: *const ShiroRsBytes,
    pub dimensions: usize,
    pub kind: u32,
}
/// Independent complete native utterance result, including every stage artifact.
pub struct ShiroRsUtterances {
    pub(super) value: SegmentedUtterances,
}
/// Independent complete audio, features and utterance workflow result.
pub struct ShiroRsSegmentedWave {
    pub(super) value: SegmentedWave,
}

/// Copy every native utterance default.
/// # Safety
/// Output is independent aligned writable initialized descriptor storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterance_options_default(
    output: *mut ShiroRsUtteranceOptions,
) -> u32 {
    // SAFETY: Independent output under the caller's contract.
    unsafe { result(output, || Ok(Options::default().into())) }
}
/// Construct all ten public fields, without rebuilding or normalizing artifacts.
/// # Safety
/// Input is initialized aligned readable storage with live immutable required
/// and optional owners. Output is independent aligned writable storage holding
/// no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_create(
    input: *const ShiroRsUtterancesInput,
    output: *mut *mut ShiroRsUtterances,
) -> u32 {
    if let Err(status) = range(input, 1) {
        return status;
    }
    // SAFETY: Complete initialized readable descriptor.
    let input = unsafe { input.read() };
    for status in [
        range(input.phonemap, 1),
        range(input.definition, 1),
        range(input.phones, 1),
        range(input.initial_segmentation, 1),
        range(input.model, 1),
        range(input.iterations, 1),
        range(input.alignment, 1),
        range(input.labels, 1),
        range(
            input.uninitialized_model,
            usize::from(!input.uninitialized_model.is_null()),
        ),
        range(
            input.initialized_model,
            usize::from(!input.initialized_model.is_null()),
        ),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable nested owners and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsUtterances {
                value: SegmentedUtterances {
                    phonemap: (*input.phonemap).value.clone(),
                    definition: (*input.definition).value.clone(),
                    phones: (*input.phones).values.clone(),
                    initial_segmentation: (*input.initial_segmentation).value.clone(),
                    uninitialized_model: if input.uninitialized_model.is_null() {
                        None
                    } else {
                        Some((*input.uninitialized_model).value.clone())
                    },
                    initialized_model: if input.initialized_model.is_null() {
                        None
                    } else {
                        Some((*input.initialized_model).value.clone())
                    },
                    model: (*input.model).value.clone(),
                    iterations: (*input.iterations).values.clone(),
                    alignment: (*input.alignment).value.clone(),
                    labels: (*input.labels).value.clone(),
                },
            })))
        })
    }
}
/// Retrieve stage presence: zero uninitialized, one initialized, two final.
/// # Safety
/// Input is live immutable ownership; output is independent aligned writable
/// storage. Invalid stage retains output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_has_model(
    utterances: *const ShiroRsUtterances,
    stage: u32,
    output: *mut u32,
) -> u32 {
    if let Err(status) = range(utterances, 1) {
        return status;
    }
    if stage > 2 {
        return 2;
    }
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        result(output, || {
            Ok(match stage {
                0 => u32::from((*utterances).value.uninitialized_model.is_some()),
                1 => u32::from((*utterances).value.initialized_model.is_some()),
                _ => 1,
            })
        })
    }
}
/// Snapshot a complete stage model: zero uninitialized, one initialized, two
/// final. Absent models or invalid stage return two without changing output.
/// # Safety
/// Input is live immutable ownership; output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_get_model(
    utterances: *const ShiroRsUtterances,
    stage: u32,
    output: *mut *mut ShiroRsModel,
) -> u32 {
    if let Err(status) = range(utterances, 1) {
        return status;
    }
    // SAFETY: Live immutable owner.
    let value = unsafe { &(*utterances).value };
    let model = match stage {
        0 => value.uninitialized_model.as_ref(),
        1 => value.initialized_model.as_ref(),
        2 => Some(&value.model),
        _ => None,
    };
    let Some(model) = model else {
        return 2;
    };
    // SAFETY: Immutable checked model and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsModel {
                value: model.clone(),
            })))
        })
    }
}
/// Run the complete native feature-to-utterance workflow, including all stages.
/// # Safety
/// Owners and initialized options/source descriptors are live and immutable.
/// Output is independent aligned writable storage holding no live owner on
/// success. Failed output remains unchanged. Models stay live throughout the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_split_features(
    features: *const ShiroRsFeatures,
    filename: *const ShiroRsBytes,
    options: *const ShiroRsUtteranceOptions,
    source: *const ShiroRsModelSource,
    output: *mut *mut ShiroRsUtterances,
) -> u32 {
    for status in [
        range(features, 1),
        range(filename, 1),
        range(options, 1),
        range(source, 1),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Complete initialized readable source descriptor.
    let source = unsafe { source.read() };
    if let Err(status) = source.validate() {
        return status;
    }
    // SAFETY: Live immutable owners and descriptors, independent output.
    unsafe {
        result(output, || {
            let filename = std::str::from_utf8(&(*filename).values).map_err(io::Error::other)?;
            let value = utterances::split_features(
                &(*features).value,
                filename,
                (*options).into(),
                source.native(),
            )?;
            Ok(Box::into_raw(Box::new(ShiroRsUtterances { value })))
        })
    }
}
/// Run complete native decoded-wave extraction and utterance segmentation.
/// # Safety
/// Input/options/source descriptors are initialized aligned readable storage;
/// nested owners and optional models remain live immutable throughout the call.
/// Callback/context obey ShiroRsUniformCallback. Output is independent aligned
/// writable storage holding no live owner on success. Failed output remains
/// unchanged; already consumed random draws are not rolled back.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_split_wave(
    input: *const ShiroRsWaveSplitInput,
    options: *const ShiroRsUtteranceOptions,
    source: *const ShiroRsModelSource,
    callback: ShiroRsUniformCallback,
    context: *mut c_void,
    output: *mut *mut ShiroRsSegmentedWave,
) -> u32 {
    for status in [range(input, 1), range(options, 1), range(source, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Complete initialized readable descriptors.
    let input = unsafe { input.read() };
    // SAFETY: Complete initialized readable descriptor.
    let source = unsafe { source.read() };
    for status in [
        range(input.samples, 1),
        range(input.filename, 1),
        source.validate(),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    let encoding = match input.header.encoding {
        0 => Encoding::Pcm,
        1 => Encoding::Float,
        _ => return 2,
    };
    let kind = match input.kind {
        0 => FeatureKind::Mfcc,
        1 => FeatureKind::Mfbe,
        2 => FeatureKind::Plpcc,
        _ => return 2,
    };
    // SAFETY: Live immutable owners, synchronous non-unwinding callback and
    // independent output validated before any native work or random draws.
    unsafe {
        result(output, || {
            let filename =
                std::str::from_utf8(&(*input.filename).values).map_err(io::Error::other)?;
            let wave = Wave {
                sample_rate: input.header.sample_rate,
                bits_per_sample: input.header.bits_per_sample,
                channels: input.header.channels,
                encoding,
                samples: (*input.samples).values.clone(),
            };
            let value = utterances::split_wave(
                wave,
                filename,
                input.dimensions,
                kind,
                (*options).into(),
                source.native(),
                || {
                    let mut draw = f32::NAN;
                    match callback {
                        Some(callback) if callback(context, &mut draw) == 0 => draw,
                        _ => f32::NAN,
                    }
                },
            )?;
            Ok(Box::into_raw(Box::new(ShiroRsSegmentedWave { value })))
        })
    }
}
/// Construct every native segmented-wave field without recomputing artifacts.
/// # Safety
/// Inputs are live immutable owners. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_wave_create(
    audio: *const ShiroRsAudio,
    features: *const ShiroRsFeatures,
    utterances: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsSegmentedWave,
) -> u32 {
    for status in [range(audio, 1), range(features, 1), range(utterances, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live immutable inputs and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentedWave {
                value: SegmentedWave {
                    audio: (*audio).value.clone(),
                    features: (*features).value.clone(),
                    utterances: (*utterances).value.clone(),
                },
            })))
        })
    }
}

/// Snapshot the complete native phonemap field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_get_phonemap(
    owner: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsPhoneMap,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsPhoneMap {
                value: (*owner).value.phonemap.clone(),
            })))
        })
    }
}

/// Snapshot the complete native definition field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_get_definition(
    owner: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsModelDefinition,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsModelDefinition {
                value: (*owner).value.definition.clone(),
            })))
        })
    }
}

/// Snapshot the complete native phones field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_get_phones(
    owner: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsStrings,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsStrings {
                values: (*owner).value.phones.clone(),
            })))
        })
    }
}

/// Snapshot the complete native initial_segmentation field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_get_initial_segmentation(
    owner: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsSegmentationDocument,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentationDocument {
                value: (*owner).value.initial_segmentation.clone(),
            })))
        })
    }
}

/// Snapshot the complete native iterations field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_get_iterations(
    owner: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsIterationReports,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsIterationReports {
                values: (*owner).value.iterations.clone(),
            })))
        })
    }
}

/// Snapshot the complete native alignment field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_get_alignment(
    owner: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsSegmentationDocument,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentationDocument {
                value: (*owner).value.alignment.clone(),
            })))
        })
    }
}

/// Snapshot the complete native labels field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_get_labels(
    owner: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsLabels,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsLabels {
                value: (*owner).value.labels.clone(),
            })))
        })
    }
}

/// Deep-copy every native public field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_clone(
    owner: *const ShiroRsUtterances,
    output: *mut *mut ShiroRsUtterances,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsUtterances {
                value: (*owner).value.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_utterances_release(slot: *mut *mut ShiroRsUtterances) -> u32 {
    // SAFETY: Unique ownership transfer under the public contract.
    unsafe { release(slot) }
}

/// Snapshot the complete native audio field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_wave_get_audio(
    owner: *const ShiroRsSegmentedWave,
    output: *mut *mut ShiroRsAudio,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsAudio {
                value: (*owner).value.audio.clone(),
            })))
        })
    }
}

/// Snapshot the complete native features field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_wave_get_features(
    owner: *const ShiroRsSegmentedWave,
    output: *mut *mut ShiroRsFeatures,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsFeatures {
                value: (*owner).value.features.clone(),
            })))
        })
    }
}

/// Snapshot the complete native utterances field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_wave_get_utterances(
    owner: *const ShiroRsSegmentedWave,
    output: *mut *mut ShiroRsUtterances,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsUtterances {
                value: (*owner).value.utterances.clone(),
            })))
        })
    }
}

/// Deep-copy every native public field into independent ownership.
/// # Safety
/// Input is a live immutable owner. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_wave_clone(
    owner: *const ShiroRsSegmentedWave,
    output: *mut *mut ShiroRsSegmentedWave,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsSegmentedWave {
                value: (*owner).value.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_segmented_wave_release(
    slot: *mut *mut ShiroRsSegmentedWave,
) -> u32 {
    // SAFETY: Unique ownership transfer under the public contract.
    unsafe { release(slot) }
}
