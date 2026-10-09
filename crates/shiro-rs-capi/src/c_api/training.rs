//! Complete ordered training inputs, results and synchronous progress reports.
use super::{
    ShiroRsArrayF32, ShiroRsBytes, ShiroRsDataset, ShiroRsModel, boundary,
    buffers::{input, release},
    range, result,
};
use crate::{
    dataset,
    labels::SegmentationDocument,
    training::{self, DurationMode, IterationReport, Options, TrainingResult},
};
use shiro_rs::hsmm::{Dataset, GeometricOptions, HsmmOptions};
use std::{ffi::c_void, io};

/// Independent ordered datasets, one per original file; each retains its groups.
pub struct ShiroRsTrainingFiles {
    pub(super) value: Vec<Dataset>,
}
/// Independent complete trained model and every ordered iteration report.
pub struct ShiroRsTrainingResult {
    pub(super) value: TrainingResult,
}

/// Encode every file likelihood row in the original CLI CSV format.
/// # Safety
/// Owner is live readable storage. Output is independent aligned writable
/// storage holding no live owner on success; failure retains its value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_result_likelihood_csv_bytes(
    owner: *const ShiroRsTrainingResult,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(owner, 1) {
        return status;
    }
    // SAFETY: Live immutable owner and independent output storage.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            (*owner).value.write_likelihood_csv(&mut values)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}
/// Complete report. Callback reports are borrowed only for the callback; clone
/// before retaining one. Only constructor/getter/clone results may be released.
pub struct ShiroRsIterationReport {
    pub(super) value: IterationReport,
}

/// All native training options. Flags/mode must be0/1; native numeric validation
/// and per-iteration annealing temperature replacement remain unchanged.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiroRsTrainingOptions {
    pub iterations: usize,
    /// 0 normal HSMM durations,1 geometric HMM durations.
    pub duration_mode: u32,
    pub hsmm_temperature: f32,
    pub duration_weight: f32,
    pub state_radius: f32,
    pub duration_extra: usize,
    pub duration_extra_factor: f32,
    pub geometric_temperature: f32,
    pub pruning_slope: f32,
    pub termination_threshold: f32,
    pub deterministic_annealing: u32,
    pub mean_frame_likelihood: u32,
    pub workers: usize,
}
/// Exact native scalar report fields; nested file rows are retrieved separately.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiroRsIterationInfo {
    pub iteration: usize,
    pub temperature: f32,
    pub mean_log_likelihood: f32,
}

// Check both the pointer table and every borrowed owner before any copy/update.
unsafe fn owners<T>(values: *const *const T, count: usize) -> Result<(), u32> {
    range(values, count)?;
    // SAFETY: Caller supplies the readable table validated above.
    for &value in unsafe { input(values, count) } {
        range(value, 1)?;
    }
    Ok(())
}

unsafe fn copy_report(
    info: *const ShiroRsIterationInfo,
    files: *const *const ShiroRsArrayF32,
    count: usize,
) -> io::Result<IterationReport> {
    let mut file_likelihoods = Vec::new();
    file_likelihoods
        .try_reserve_exact(count)
        .map_err(io::Error::other)?;
    // SAFETY: Public entry point checked the descriptor, table and live owners.
    let info = unsafe { info.read() };
    // SAFETY: Every table entry points to a live immutable array owner.
    for &file in unsafe { input(files, count) } {
        file_likelihoods.push(unsafe { &(*file).values }.clone());
    }
    Ok(IterationReport {
        iteration: info.iteration,
        temperature: info.temperature,
        mean_log_likelihood: info.mean_log_likelihood,
        file_likelihoods,
    })
}

unsafe fn copy_training_result(
    model: *const ShiroRsModel,
    reports: *const *const ShiroRsIterationReport,
    count: usize,
) -> io::Result<TrainingResult> {
    let mut iterations = Vec::new();
    iterations
        .try_reserve_exact(count)
        .map_err(io::Error::other)?;
    // SAFETY: Public entry point checked the table and every live report owner.
    for &report in unsafe { input(reports, count) } {
        iterations.push(unsafe { &(*report).value }.clone());
    }
    Ok(TrainingResult {
        // SAFETY: Validated live immutable model owner.
        model: unsafe { &(*model).value }.clone(),
        iterations,
    })
}

/// Replace every report field after copying all inputs. Failure retains the owner.
/// # Safety
/// Owner is live exclusively writable owned storage, never a callback-borrowed
/// report. Info/table/array owners obey create's readable contract and do not
/// overlap the destination. Inputs remain unchanged and are not retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_report_replace(
    owner: *mut ShiroRsIterationReport,
    info: *const ShiroRsIterationInfo,
    files: *const *const ShiroRsArrayF32,
    count: usize,
) -> u32 {
    // SAFETY: Caller supplies readable table storage when its range is valid.
    for status in [range(owner, 1), range(info, 1), unsafe {
        owners(files, count)
    }] {
        if let Err(status) = status {
            return status;
        }
    }
    boundary(|| {
        // SAFETY: Checked immutable sources and exclusive destination.
        let value = unsafe { copy_report(info, files, count) }?;
        unsafe {
            (*owner).value = value;
        }
        Ok(())
    })
    .map_or_else(|status| status, |()| 0)
}

/// Copy the complete model and arbitrary ordered reports without training.
/// # Safety
/// Model, report table and every report are live aligned readable storage; null
/// table is allowed only for zero count. Output is independent writable storage
/// holding no live owner on success; failed output and inputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_result_create(
    model: *const ShiroRsModel,
    reports: *const *const ShiroRsIterationReport,
    count: usize,
    output: *mut *mut ShiroRsTrainingResult,
) -> u32 {
    // SAFETY: Caller supplies readable table storage when its range is valid.
    for status in [range(model, 1), unsafe { owners(reports, count) }] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Checked immutable sources and independent writable output.
    unsafe {
        result(output, || {
            let value = copy_training_result(model, reports, count)?;
            Ok(Box::into_raw(Box::new(ShiroRsTrainingResult { value })))
        })
    }
}

/// Replace both native result fields only after copying the complete inputs.
/// # Safety
/// Owner is live exclusively writable storage. Model/table/report owners obey
/// create's readable contract and do not overlap the destination. Inputs are
/// copied, not retained; failure leaves every existing result field unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_result_replace(
    owner: *mut ShiroRsTrainingResult,
    model: *const ShiroRsModel,
    reports: *const *const ShiroRsIterationReport,
    count: usize,
) -> u32 {
    // SAFETY: Caller supplies readable table storage when its range is valid.
    for status in [range(owner, 1), range(model, 1), unsafe {
        owners(reports, count)
    }] {
        if let Err(status) = status {
            return status;
        }
    }
    boundary(|| {
        // SAFETY: Checked immutable sources and exclusive destination.
        let value = unsafe { copy_training_result(model, reports, count) }?;
        unsafe {
            (*owner).value = value;
        }
        Ok(())
    })
    .map_or_else(|status| status, |()| 0)
}
/// Synchronous progress callback. Report is read-only and live only for this call.
/// It may be queried or cloned, but must not be released or retained un-cloned.
/// Callback must return normally without exceptions/unwinding across the C ABI;
/// participating training input owners must not be mutated or released.
pub type ShiroRsProgressCallback =
    Option<unsafe extern "C" fn(*mut c_void, *const ShiroRsIterationReport)>;

impl From<Options> for ShiroRsTrainingOptions {
    fn from(value: Options) -> Self {
        Self {
            iterations: value.iterations,
            duration_mode: match value.duration_mode {
                DurationMode::Normal => 0,
                DurationMode::Geometric => 1,
            },
            hsmm_temperature: value.hsmm.temperature,
            duration_weight: value.hsmm.duration_weight,
            state_radius: value.hsmm.state_radius,
            duration_extra: value.hsmm.duration_extra,
            duration_extra_factor: value.hsmm.duration_extra_factor,
            geometric_temperature: value.geometric.temperature,
            pruning_slope: value.geometric.pruning_slope,
            termination_threshold: value.termination_threshold,
            deterministic_annealing: u32::from(value.deterministic_annealing),
            mean_frame_likelihood: u32::from(value.mean_frame_likelihood),
            workers: value.workers,
        }
    }
}
impl TryFrom<ShiroRsTrainingOptions> for Options {
    type Error = u32;
    fn try_from(value: ShiroRsTrainingOptions) -> Result<Self, Self::Error> {
        let flag = |value| match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(2u32),
        };
        let duration_mode = match value.duration_mode {
            0 => DurationMode::Normal,
            1 => DurationMode::Geometric,
            _ => return Err(2),
        };
        Ok(Self {
            iterations: value.iterations,
            duration_mode,
            hsmm: HsmmOptions {
                temperature: value.hsmm_temperature,
                duration_weight: value.duration_weight,
                state_radius: value.state_radius,
                duration_extra: value.duration_extra,
                duration_extra_factor: value.duration_extra_factor,
            },
            geometric: GeometricOptions {
                temperature: value.geometric_temperature,
                pruning_slope: value.pruning_slope,
            },
            termination_threshold: value.termination_threshold,
            deterministic_annealing: flag(value.deterministic_annealing)?,
            mean_frame_likelihood: flag(value.mean_frame_likelihood)?,
            workers: value.workers,
        })
    }
}

/// Clone complete paired datasets into independent file order. Repeated and empty
/// inputs are permitted; native training validates their applicability.
/// # Safety
/// Array is aligned initialized readable storage of count live owners from this
/// library. Output is independent aligned writable storage holding no live owner
/// on success. Inputs and failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_files_create(
    files: *const *const ShiroRsDataset,
    count: usize,
    output: *mut *mut ShiroRsTrainingFiles,
) -> u32 {
    if let Err(status) = range(files, count) {
        return status;
    }
    // SAFETY: Checked readable array under the public contract.
    let files = unsafe { input(files, count) };
    for &file in files {
        if let Err(status) = range(file, 1) {
            return status;
        }
    }
    // SAFETY: All borrowed owners are readable and output is independent.
    unsafe {
        result(output, || {
            let mut value = Vec::new();
            value.try_reserve_exact(count).map_err(io::Error::other)?;
            for &file in files {
                value.push((*file).value.clone());
            }
            Ok(Box::into_raw(Box::new(ShiroRsTrainingFiles { value })))
        })
    }
}

/// Load one complete paired dataset per original file, preserving isolated groups
/// and file order via the unchanged native host loader. isolated must be0/1.
/// # Safety
/// Inputs are live readable owners. Output is independent aligned writable storage
/// holding no live owner on success. Inputs and failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_files_read_document(
    model: *const ShiroRsModel,
    document: *const ShiroRsBytes,
    maximum_frames: usize,
    isolated: u32,
    output: *mut *mut ShiroRsTrainingFiles,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    if let Err(status) = range(document, 1) {
        return status;
    }
    let isolated = match isolated {
        0 => false,
        1 => true,
        _ => return 2,
    };
    // SAFETY: Live readable owners and independent output.
    unsafe {
        result(output, || {
            let document: SegmentationDocument =
                serde_json::from_slice(&(*document).values).map_err(io::Error::other)?;
            let value =
                dataset::load_training_files(&document, &(*model).value, maximum_frames, isolated)?;
            Ok(Box::into_raw(Box::new(ShiroRsTrainingFiles { value })))
        })
    }
}

/// Retrieve the ordered file count.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_files_length(
    files: *const ShiroRsTrainingFiles,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(files, 1) {
        return status;
    }
    // SAFETY: Live readable input and independent output.
    unsafe { result(output, || Ok((*files).value.len())) }
}

/// Deep-copy one complete paired dataset, retaining all groups and scalar bits.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_files_get_dataset(
    files: *const ShiroRsTrainingFiles,
    index: usize,
    output: *mut *mut ShiroRsDataset,
) -> u32 {
    if let Err(status) = range(files, 1) {
        return status;
    }
    // SAFETY: Live readable owner.
    let Some(value) = (unsafe { &(*files).value }).get(index) else {
        return 2;
    };
    // SAFETY: Readable source and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsDataset {
                value: value.clone(),
            })))
        })
    }
}

/// Deep-copy every ordered file and group.
/// # Safety
/// Input is a live readable owner; output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_files_clone(
    files: *const ShiroRsTrainingFiles,
    output: *mut *mut ShiroRsTrainingFiles,
) -> u32 {
    if let Err(status) = range(files, 1) {
        return status;
    }
    // SAFETY: Live readable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsTrainingFiles {
                value: (*files).value.clone(),
            })))
        })
    }
}

/// Release unique files and clear their slot; empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership to this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_files_release(
    slot: *mut *mut ShiroRsTrainingFiles,
) -> u32 {
    // SAFETY: Unique owner transfer under public contract.
    unsafe { release(slot) }
}

/// Copy every unchanged native training default into the descriptor.
/// # Safety
/// Output is independent aligned exclusively writable descriptor storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_options_default(
    output: *mut ShiroRsTrainingOptions,
) -> u32 {
    // SAFETY: Independent output storage under the public contract.
    unsafe { result(output, || Ok(Options::default().into())) }
}

unsafe fn train(
    model: *const ShiroRsModel,
    files: *const ShiroRsTrainingFiles,
    options: *const ShiroRsTrainingOptions,
    callback: ShiroRsProgressCallback,
    context: *mut c_void,
    output: *mut *mut ShiroRsTrainingResult,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    if let Err(status) = range(files, 1) {
        return status;
    }
    if let Err(status) = range(options, 1) {
        return status;
    }
    // SAFETY: Initialized readable descriptor storage.
    let options = match Options::try_from(unsafe { options.read() }) {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: Live immutable input owners; callback obeys its synchronous contract.
    unsafe {
        result(output, || {
            let value = match callback {
                Some(callback) => training::train_with_progress(
                    &(*model).value,
                    &(*files).value,
                    options,
                    |report| {
                        let borrowed = ShiroRsIterationReport {
                            value: report.clone(),
                        };
                        callback(context, &borrowed);
                    },
                ),
                None => training::train(&(*model).value, &(*files).value, options),
            }
            .map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsTrainingResult { value })))
        })
    }
}

/// Train with every native setting, retaining all reports and the complete model.
/// # Safety
/// Inputs are live readable owners and options are initialized readable descriptor
/// storage. Output is independent aligned writable storage holding no live owner
/// on success. Inputs and failed output slots remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_train(
    model: *const ShiroRsModel,
    files: *const ShiroRsTrainingFiles,
    options: *const ShiroRsTrainingOptions,
    output: *mut *mut ShiroRsTrainingResult,
) -> u32 {
    // SAFETY: Validated immutable owners and independent output under public contract.
    unsafe { train(model, files, options, None, std::ptr::null_mut(), output) }
}

/// Train with synchronous progress after each model update and before report
/// storage/stopping checks. Notifications already delivered are not rolled back
/// if a later iteration fails. Null callback means no notifications.
/// # Safety
/// Inputs/options/output obey train's contract. Callback and context remain valid
/// throughout the call and obey ShiroRsProgressCallback's borrowed-lifetime/no-unwind
/// contract. Callback must not mutate/release participating training inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_train_with_progress(
    model: *const ShiroRsModel,
    files: *const ShiroRsTrainingFiles,
    options: *const ShiroRsTrainingOptions,
    callback: ShiroRsProgressCallback,
    context: *mut c_void,
    output: *mut *mut ShiroRsTrainingResult,
) -> u32 {
    // SAFETY: Public caller supplies valid immutable inputs and synchronous callback.
    unsafe { train(model, files, options, callback, context, output) }
}

/// Retrieve the number of completed ordered iteration reports.
/// # Safety
/// Input is a live readable owner and output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_result_length(
    training: *const ShiroRsTrainingResult,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(training, 1) {
        return status;
    }
    // SAFETY: Live input and independent output.
    unsafe { result(output, || Ok((*training).value.iterations.len())) }
}

/// Deep-copy every trained model parameter into independent storage.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_result_get_model(
    training: *const ShiroRsTrainingResult,
    output: *mut *mut ShiroRsModel,
) -> u32 {
    if let Err(status) = range(training, 1) {
        return status;
    }
    // SAFETY: Live input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsModel {
                value: (*training).value.model.clone(),
            })))
        })
    }
}

/// Deep-copy a complete report, including every ordered file/group likelihood.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_result_get_report(
    training: *const ShiroRsTrainingResult,
    index: usize,
    output: *mut *mut ShiroRsIterationReport,
) -> u32 {
    if let Err(status) = range(training, 1) {
        return status;
    }
    // SAFETY: Live readable owner.
    let Some(value) = (unsafe { &(*training).value.iterations }).get(index) else {
        return 2;
    };
    // SAFETY: Readable source and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsIterationReport {
                value: value.clone(),
            })))
        })
    }
}

/// Deep-copy the complete model and every nested report field.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_result_clone(
    training: *const ShiroRsTrainingResult,
    output: *mut *mut ShiroRsTrainingResult,
) -> u32 {
    if let Err(status) = range(training, 1) {
        return status;
    }
    // SAFETY: Live input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsTrainingResult {
                value: (*training).value.clone(),
            })))
        })
    }
}

/// Release a unique training result and clear its slot; empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null. Release requires exclusive access and transfers ownership to this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_training_result_release(
    slot: *mut *mut ShiroRsTrainingResult,
) -> u32 {
    // SAFETY: Unique owner transfer under public contract.
    unsafe { release(slot) }
}

/// Copy all exact native scalar report fields. Callback reports may be queried.
/// # Safety
/// Input is a live readable owned report or a live callback-borrowed report.
/// Output is independent aligned writable descriptor storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_report_info(
    report: *const ShiroRsIterationReport,
    output: *mut ShiroRsIterationInfo,
) -> u32 {
    if let Err(status) = range(report, 1) {
        return status;
    }
    // SAFETY: Live input and independent output.
    unsafe {
        result(output, || {
            Ok(ShiroRsIterationInfo {
                iteration: (*report).value.iteration,
                temperature: (*report).value.temperature,
                mean_log_likelihood: (*report).value.mean_log_likelihood,
            })
        })
    }
}

/// Retrieve the original file count, retaining nested group boundaries.
/// # Safety
/// Input is a live readable owned/callback-borrowed report. Output is independent
/// aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_report_file_count(
    report: *const ShiroRsIterationReport,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(report, 1) {
        return status;
    }
    // SAFETY: Live input and independent output.
    unsafe { result(output, || Ok((*report).value.file_likelihoods.len())) }
}

/// Copy the complete binary32 group likelihood row for one original file.
/// # Safety
/// Input is a live readable owned/callback-borrowed report. Output is independent
/// aligned writable storage holding no live owner on success; failures retain it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_report_get_file(
    report: *const ShiroRsIterationReport,
    index: usize,
    output: *mut *mut ShiroRsArrayF32,
) -> u32 {
    if let Err(status) = range(report, 1) {
        return status;
    }
    // SAFETY: Live readable report.
    let Some(values) = (unsafe { &(*report).value.file_likelihoods }).get(index) else {
        return 2;
    };
    // SAFETY: Readable source and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsArrayF32 {
                values: values.clone(),
            })))
        })
    }
}

/// Clone a complete owned or callback-borrowed report into independent ownership.
/// # Safety
/// Input is a live readable owned/callback-borrowed report. Output is independent
/// aligned writable storage holding no live owner on success; failures retain it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_report_clone(
    report: *const ShiroRsIterationReport,
    output: *mut *mut ShiroRsIterationReport,
) -> u32 {
    if let Err(status) = range(report, 1) {
        return status;
    }
    // SAFETY: Live input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsIterationReport {
                value: (*report).value.clone(),
            })))
        })
    }
}

/// Release a unique owned report and clear its slot; never release a callback's
/// borrowed report. Empty slot succeeds.
/// # Safety
/// Slot holds a unique owned report from a constructor/getter/clone or null; aligned independent
/// writable storage and exclusive access are required. Callback reports are excluded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_report_release(
    slot: *mut *mut ShiroRsIterationReport,
) -> u32 {
    // SAFETY: Unique owned report transfer under public contract.
    unsafe { release(slot) }
}
