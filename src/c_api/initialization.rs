//! Independent complete paired datasets and the unchanged native initializer.
use super::{
    ShiroRsBytes, ShiroRsModel, ShiroRsObservation, ShiroRsStates,
    buffers::{input, release},
    range, result,
};
use crate::{
    dataset,
    initialization::{self, Options},
    labels::SegmentationDocument,
};
use liblrhsmm_rs::Dataset;
use std::io;

/// Independent paired native observations and segmentations in sample order.
pub struct ShiroRsDataset {
    pub(super) value: Dataset,
}

/// All native initializer settings. Flags must be 0 or 1.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShiroRsInitializationOptions {
    pub flat_start: u32,
    pub globally_tied: u32,
    pub variance_floor_ratio: f32,
}
impl From<Options> for ShiroRsInitializationOptions {
    fn from(value: Options) -> Self {
        Self {
            flat_start: u32::from(value.flat_start),
            globally_tied: u32::from(value.globally_tied),
            variance_floor_ratio: value.variance_floor_ratio,
        }
    }
}
impl TryFrom<ShiroRsInitializationOptions> for Options {
    type Error = u32;
    fn try_from(value: ShiroRsInitializationOptions) -> Result<Self, Self::Error> {
        let flag = |value| match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(2u32),
        };
        Ok(Self {
            flat_start: flag(value.flat_start)?,
            globally_tied: flag(value.globally_tied)?,
            variance_floor_ratio: value.variance_floor_ratio,
        })
    }
}

/// Deep-copy all observations and convert all state sequences using the model.
/// Repeated owners are allowed; empty inputs create an empty paired dataset.
/// # Safety
/// Model and every array element are live readable owners from this library.
/// Arrays are aligned initialized readable storage of sample_count elements.
/// Output is independent aligned writable storage holding no live owner on success.
/// Inputs remain unchanged; failed calls retain the initialized output slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dataset_create(
    model: *const ShiroRsModel,
    observations: *const *const ShiroRsObservation,
    states: *const *const ShiroRsStates,
    sample_count: usize,
    output: *mut *mut ShiroRsDataset,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    if let Err(status) = range(observations, sample_count) {
        return status;
    }
    if let Err(status) = range(states, sample_count) {
        return status;
    }
    // SAFETY: Both validated arrays are initialized readable storage.
    let observations = unsafe { input(observations, sample_count) };
    // SAFETY: Both validated arrays are initialized readable storage.
    let states = unsafe { input(states, sample_count) };
    for (&observation, &states) in observations.iter().zip(states) {
        if let Err(status) = range(observation, 1) {
            return status;
        }
        if let Err(status) = range(states, 1) {
            return status;
        }
    }
    // SAFETY: All borrowed owners are readable and output is independent.
    unsafe {
        result(output, || {
            let mut value = Dataset::default();
            value
                .observations
                .try_reserve_exact(sample_count)
                .map_err(io::Error::other)?;
            value
                .segmentations
                .try_reserve_exact(sample_count)
                .map_err(io::Error::other)?;
            for (&observation, &states) in observations.iter().zip(states) {
                value.segmentations.push(dataset::read_segmentation(
                    &(*states).value,
                    &(*model).value,
                )?);
                value.observations.push((*observation).value.clone());
            }
            Ok(Box::into_raw(Box::new(ShiroRsDataset { value })))
        })
    }
}

/// Load a complete original JSON document with unchanged native filename/frame
/// semantics. All samples remain paired in document order.
/// # Safety
/// Inputs are live readable owners. Output is independent aligned writable storage
/// holding no live owner on success. Inputs and failed output slots remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dataset_read_document(
    model: *const ShiroRsModel,
    document: *const ShiroRsBytes,
    maximum_frames: usize,
    output: *mut *mut ShiroRsDataset,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    if let Err(status) = range(document, 1) {
        return status;
    }
    // SAFETY: Live readable input owners and independent output slot.
    unsafe {
        result(output, || {
            let document: SegmentationDocument =
                serde_json::from_slice(&(*document).values).map_err(io::Error::other)?;
            let value = dataset::load(&document, &(*model).value, maximum_frames)?;
            Ok(Box::into_raw(Box::new(ShiroRsDataset { value })))
        })
    }
}

/// Retrieve the paired sample count without borrowing internal storage.
/// # Safety
/// Input is a live readable owner and output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dataset_length(
    dataset: *const ShiroRsDataset,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(dataset, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output slot.
    unsafe { result(output, || Ok((*dataset).value.observations.len())) }
}

/// Retrieve a complete independent observation snapshot by sample index.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dataset_get_observation(
    dataset: *const ShiroRsDataset,
    index: usize,
    output: *mut *mut ShiroRsObservation,
) -> u32 {
    if let Err(status) = range(dataset, 1) {
        return status;
    }
    // SAFETY: Live readable owner under the public contract.
    let Some(value) = (unsafe { &(*dataset).value.observations }).get(index) else {
        return 2;
    };
    // SAFETY: Live readable snapshot source and independent output slot.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsObservation {
                value: value.clone(),
            })))
        })
    }
}

/// Serialize every field of one complete segmentation in the original format.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dataset_get_segmentation_bytes(
    dataset: *const ShiroRsDataset,
    index: usize,
    output: *mut *mut ShiroRsBytes,
) -> u32 {
    if let Err(status) = range(dataset, 1) {
        return status;
    }
    // SAFETY: Live readable owner under the public contract.
    let Some(value) = (unsafe { &(*dataset).value.segmentations }).get(index) else {
        return 2;
    };
    // SAFETY: Live readable snapshot source and independent output slot.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            value.write_to(&mut values)?;
            Ok(Box::into_raw(Box::new(ShiroRsBytes { values })))
        })
    }
}

/// Deep-copy the complete paired dataset, retaining every scalar bit.
/// # Safety
/// Input is a live readable owner. Output is independent aligned writable storage
/// holding no live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dataset_clone(
    dataset: *const ShiroRsDataset,
    output: *mut *mut ShiroRsDataset,
) -> u32 {
    if let Err(status) = range(dataset, 1) {
        return status;
    }
    // SAFETY: Live readable owner and independent output slot.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsDataset {
                value: (*dataset).value.clone(),
            })))
        })
    }
}

/// Release a unique dataset and clear its slot. An empty slot succeeds.
/// # Safety
/// Slot is independent aligned initialized writable storage holding a unique live
/// owner from this library or null. Release requires exclusive access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_dataset_release(slot: *mut *mut ShiroRsDataset) -> u32 {
    // SAFETY: Public contract transfers unique ownership through the slot.
    unsafe { release(slot) }
}

/// Copy the unchanged native initializer defaults into caller storage.
/// # Safety
/// Output is independent aligned exclusively writable descriptor storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_initialization_options_default(
    output: *mut ShiroRsInitializationOptions,
) -> u32 {
    // SAFETY: Independent validated descriptor output under the public contract.
    unsafe { result(output, || Ok(Options::default().into())) }
}

/// Initialize all model parameters from the complete paired dataset. Inputs are
/// never modified; native corpus fallback/tied/flat rounding corrections apply.
/// # Safety
/// Inputs are live readable owners and options are aligned initialized readable
/// descriptor storage. Output is independent aligned writable storage holding no
/// live owner on success. Failed outputs remain unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_initialize(
    model: *const ShiroRsModel,
    dataset: *const ShiroRsDataset,
    options: *const ShiroRsInitializationOptions,
    output: *mut *mut ShiroRsModel,
) -> u32 {
    if let Err(status) = range(model, 1) {
        return status;
    }
    if let Err(status) = range(dataset, 1) {
        return status;
    }
    if let Err(status) = range(options, 1) {
        return status;
    }
    // SAFETY: Caller supplies initialized readable descriptor storage.
    let options = match Options::try_from(unsafe { options.read() }) {
        Ok(value) => value,
        Err(status) => return status,
    };
    // SAFETY: Live readable input owners and independent output slot.
    unsafe {
        result(output, || {
            let value = initialization::initialize(&(*model).value, &(*dataset).value, options)
                .map_err(io::Error::other)?;
            Ok(Box::into_raw(Box::new(ShiroRsModel { value })))
        })
    }
}
