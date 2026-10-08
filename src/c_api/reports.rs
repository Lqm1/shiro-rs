//! Complete ordered iteration reports without reducing nested likelihood rows.
use super::{
    ShiroRsArrayF32, ShiroRsIterationInfo, ShiroRsIterationReport,
    buffers::{input, release},
    range, result,
};
use crate::training::IterationReport;
use std::io;

/// Independent ordered native report collection, including every nested row.
pub struct ShiroRsIterationReports {
    pub(super) values: Vec<IterationReport>,
}

/// Construct every native report field, retaining all binary32 bits and empty rows.
/// Repeated immutable row owners are permitted; no training validation is imposed.
/// # Safety
/// Info is initialized aligned readable storage. Rows is an initialized readable
/// pointer range for count live immutable array owners; empty input permits null.
/// Output is independent aligned writable storage holding no live owner on success.
/// Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_report_create(
    info: *const ShiroRsIterationInfo,
    rows: *const *const ShiroRsArrayF32,
    count: usize,
    output: *mut *mut ShiroRsIterationReport,
) -> u32 {
    for status in [range(info, 1), range(rows, count)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Complete initialized readable pointer range.
    let rows = unsafe { input(rows, count) };
    for &row in rows {
        if let Err(status) = range(row, 1) {
            return status;
        }
    }
    // SAFETY: Live immutable descriptors and array owners, independent output.
    unsafe {
        result(output, || {
            let mut file_likelihoods = Vec::new();
            file_likelihoods
                .try_reserve_exact(count)
                .map_err(io::Error::other)?;
            for &row in rows {
                file_likelihoods.push((*row).values.clone());
            }
            Ok(Box::into_raw(Box::new(ShiroRsIterationReport {
                value: IterationReport {
                    iteration: (*info).iteration,
                    temperature: (*info).temperature,
                    mean_log_likelihood: (*info).mean_log_likelihood,
                    file_likelihoods,
                },
            })))
        })
    }
}

/// Copy complete reports in input order; repeated immutable owners are permitted.
/// # Safety
/// Input is aligned readable pointer storage for count live immutable report
/// owners; empty input permits null. Output is independent aligned writable
/// storage holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_reports_create(
    reports: *const *const ShiroRsIterationReport,
    count: usize,
    output: *mut *mut ShiroRsIterationReports,
) -> u32 {
    if let Err(status) = range(reports, count) {
        return status;
    }
    // SAFETY: Complete initialized readable pointer range.
    let reports = unsafe { input(reports, count) };
    for &report in reports {
        if let Err(status) = range(report, 1) {
            return status;
        }
    }
    // SAFETY: Live immutable report owners and independent output.
    unsafe {
        result(output, || {
            let mut values = Vec::new();
            values.try_reserve_exact(count).map_err(io::Error::other)?;
            for &report in reports {
                values.push((*report).value.clone());
            }
            Ok(Box::into_raw(Box::new(ShiroRsIterationReports { values })))
        })
    }
}
/// Retrieve the complete ordered report count.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_reports_length(
    reports: *const ShiroRsIterationReports,
    output: *mut usize,
) -> u32 {
    if let Err(status) = range(reports, 1) {
        return status;
    }
    // SAFETY: Live readable input and independent output.
    unsafe { result(output, || Ok((*reports).values.len())) }
}
/// Snapshot every checked report field and nested row into independent ownership.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_reports_get(
    reports: *const ShiroRsIterationReports,
    index: usize,
    output: *mut *mut ShiroRsIterationReport,
) -> u32 {
    if let Err(status) = range(reports, 1) {
        return status;
    }
    // SAFETY: Live readable owner and checked index.
    let Some(report) = (unsafe { &(*reports).values }).get(index) else {
        return 2;
    };
    // SAFETY: Immutable report and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsIterationReport {
                value: report.clone(),
            })))
        })
    }
}
/// Deep-copy every ordered report field and nested row into independent ownership.
/// # Safety
/// Input is live and readable; output is independent aligned writable storage
/// holding no live owner on success. Failed output remains unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_reports_clone(
    reports: *const ShiroRsIterationReports,
    output: *mut *mut ShiroRsIterationReports,
) -> u32 {
    if let Err(status) = range(reports, 1) {
        return status;
    }
    // SAFETY: Live immutable input and independent output.
    unsafe {
        result(output, || {
            Ok(Box::into_raw(Box::new(ShiroRsIterationReports {
                values: (*reports).values.clone(),
            })))
        })
    }
}
/// Release unique ownership and clear the slot; an empty slot succeeds.
/// # Safety
/// Slot is independent aligned writable storage holding a unique live owner or
/// null; release requires exclusive access and transfers ownership.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_iteration_reports_release(
    slot: *mut *mut ShiroRsIterationReports,
) -> u32 {
    // SAFETY: Unique ownership transfer through independent initialized storage.
    unsafe { release(slot) }
}
