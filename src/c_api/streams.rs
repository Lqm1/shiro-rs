//! Direct synchronous native IO; callbacks and contexts are never retained.
use super::{
    ShiroRsArrayF32, ShiroRsIndexEntries, ShiroRsLabels, ShiroRsObservation, ShiroRsPath,
    ShiroRsStrings, ShiroRsUntiedModel, boundary, buffers::input, range, result,
};
use std::{
    ffi::c_void,
    io::{self, BufRead, Read, Write},
};

pub const SHIRO_RS_IO_SUCCESS: u32 = 0;
pub const SHIRO_RS_IO_INTERRUPTED: u32 = 1;
pub const SHIRO_RS_IO_ERROR: u32 = 2;

/// Direct reader: zero success, one interrupted, other statuses IO error.
/// Read fills at most capacity bytes and reports count; zero count is EOF.
/// Callback and independent context remain valid for the synchronous call.
/// Callbacks must not unwind, retain buffers or modify active owners/output slots.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShiroRsReadStream {
    pub context: *mut c_void,
    pub read: Option<unsafe extern "C" fn(*mut c_void, *mut u8, usize, *mut usize) -> u32>,
}

/// Direct partial writer with the reader's statuses. Count must not exceed input.
/// No native workflow implicitly flushes, closes or retains this stream.
/// Flush is optional except for explicit write_stream_flush. Callback and context
/// must satisfy the same independent, synchronous, non-unwinding contract.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShiroRsWriteStream {
    pub context: *mut c_void,
    pub write: Option<unsafe extern "C" fn(*mut c_void, *const u8, usize, *mut usize) -> u32>,
    pub flush: Option<unsafe extern "C" fn(*mut c_void) -> u32>,
}

/// Direct BufRead callbacks, with no added buffer or read-ahead.
/// Fill writes a borrowed initialized readable buffer pointer and length; empty
/// means EOF. Statuses match ReadStream. Buffer remains valid until the next
/// fill/consume. Consume is infallible and advances exactly the supplied count.
/// Callbacks/context satisfy the synchronous non-unwinding contract, and must
/// not mutate active owners or output slots. Neither callback is retained.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShiroRsBufferedReadStream {
    pub context: *mut c_void,
    pub fill: Option<unsafe extern "C" fn(*mut c_void, *mut *const u8, *mut usize) -> u32>,
    pub consume: Option<unsafe extern "C" fn(*mut c_void, usize)>,
}

fn callback_result(status: u32) -> io::Result<()> {
    match status {
        SHIRO_RS_IO_SUCCESS => Ok(()),
        SHIRO_RS_IO_INTERRUPTED => Err(io::ErrorKind::Interrupted.into()),
        _ => Err(io::Error::other("IO callback failed")),
    }
}
struct Reader(ShiroRsReadStream);
impl Read for Reader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let callback = self
            .0
            .read
            .ok_or_else(|| io::Error::other("missing read callback"))?;
        let mut count = 0;
        // SAFETY: Synchronous callback receives initialized independent output.
        callback_result(unsafe {
            callback(
                self.0.context,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut count,
            )
        })?;
        if count > buffer.len() {
            return Err(io::Error::other("read count exceeds buffer"));
        }
        Ok(count)
    }
}
struct Writer(ShiroRsWriteStream);
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let callback = self
            .0
            .write
            .ok_or_else(|| io::Error::other("missing write callback"))?;
        let mut count = 0;
        // SAFETY: Synchronous callback borrows immutable native bytes.
        callback_result(unsafe {
            callback(self.0.context, bytes.as_ptr(), bytes.len(), &mut count)
        })?;
        if count > bytes.len() {
            return Err(io::Error::other("write count exceeds input"));
        }
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        let callback = self
            .0
            .flush
            .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, "missing flush callback"))?;
        // SAFETY: Synchronous callback/context contract.
        callback_result(unsafe { callback(self.0.context) })
    }
}
struct BufferedReader(ShiroRsBufferedReadStream);
impl Read for BufferedReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        let buffer = self.fill_buf()?;
        let count = buffer.len().min(output.len());
        output[..count].copy_from_slice(&buffer[..count]);
        self.consume(count);
        Ok(count)
    }
}
impl BufRead for BufferedReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let callback = self
            .0
            .fill
            .ok_or_else(|| io::Error::other("missing fill callback"))?;
        let mut bytes = std::ptr::null();
        let mut count = 0;
        // SAFETY: Independent borrowed-buffer pointer/count slots for this callback.
        callback_result(unsafe { callback(self.0.context, &mut bytes, &mut count) })?;
        range(bytes, count).map_err(|_| io::Error::other("invalid buffered read range"))?;
        // SAFETY: Callback guarantees the complete initialized readable buffer's
        // validity until the next exclusive fill/consume call, tied to this borrow.
        Ok(unsafe { input(bytes, count) })
    }
    fn consume(&mut self, count: usize) {
        // Mandatory callback is checked before native reading starts.
        let callback = self.0.consume.expect("validated consume callback");
        // SAFETY: Native BufRead consumes only its previous valid buffer range.
        unsafe { callback(self.0.context, count) };
    }
}

/// Direct native rawfloat reader with the caller's full sample budget.
/// # Safety
/// Stream is aligned initialized metadata with a live independent context and
/// non-unwinding callback. Output is independent aligned writable storage holding
/// no live owner on success. Failure retains output; consumed input is not undone.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_rawfloat_read_stream(
    stream: *const ShiroRsReadStream,
    maximum_samples: usize,
    output: *mut *mut ShiroRsArrayF32,
) -> u32 {
    if let Err(status) = range(stream, 1) {
        return status;
    }
    // SAFETY: Live initialized immutable descriptor.
    let stream = unsafe { stream.read() };
    // SAFETY: Independent output validated before any native read/callback.
    unsafe {
        result(output, || {
            if stream.read.is_none() {
                return Err(io::Error::other("missing read callback"));
            }
            Ok(Box::into_raw(Box::new(ShiroRsArrayF32 {
                values: crate::rawfloat::read(Reader(stream), maximum_samples)?,
            })))
        })
    }
}

/// Direct native rawfloat writes retain all binary32 bits and partial transfers.
/// # Safety
/// Input is a live readable owner; stream is aligned initialized metadata with
/// an independent live context and non-unwinding callback. Already emitted bytes
/// remain after failure. No flushing or closing is performed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_rawfloat_write_stream(
    values: *const ShiroRsArrayF32,
    stream: *const ShiroRsWriteStream,
) -> u32 {
    for status in [range(values, 1), range(stream, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Initialized immutable metadata and live readable values.
    let stream = unsafe { stream.read() };
    boundary(|| {
        if stream.write.is_none() {
            return Err(io::Error::other("missing write callback"));
        }
        // SAFETY: Live immutable owner retained for native operation.
        crate::rawfloat::write(Writer(stream), unsafe { &(*values).values })
    })
    .map_or_else(|status| status, |()| 0)
}

/// Direct native observation read with every stream dimension and frame budget.
/// # Safety
/// Stream satisfies ReadStream's contract; dimensions are initialized aligned
/// readable storage of the stated count. Output is independent aligned writable
/// storage holding no live owner on success. Failure retains output, not input
/// consumption. Empty dimensions permit null and follow native validation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_observation_read_stream(
    stream: *const ShiroRsReadStream,
    dimensions: *const usize,
    stream_count: usize,
    maximum_frames: usize,
    output: *mut *mut ShiroRsObservation,
) -> u32 {
    for status in [range(stream, 1), range(dimensions, stream_count)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live initialized immutable descriptor.
    let stream = unsafe { stream.read() };
    // SAFETY: Complete dimension range and independent output.
    unsafe {
        result(output, || {
            if stream.read.is_none() {
                return Err(io::Error::other("missing read callback"));
            }
            Ok(Box::into_raw(Box::new(ShiroRsObservation {
                value: crate::dataset::read_observation(
                    Reader(stream),
                    input(dimensions, stream_count),
                    maximum_frames,
                )?,
            })))
        })
    }
}

/// Direct native label output with original CRLF and native partial-write errors.
/// # Safety
/// Input is a live immutable owner; stream satisfies WriteStream's contract.
/// Partial emitted bytes are retained on failure. No implicit flush or close.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_labels_write_stream(
    labels: *const ShiroRsLabels,
    stream: *const ShiroRsWriteStream,
) -> u32 {
    for status in [range(labels, 1), range(stream, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live initialized descriptor.
    let stream = unsafe { stream.read() };
    boundary(|| {
        if stream.write.is_none() {
            return Err(io::Error::other("missing write callback"));
        }
        // SAFETY: Live readable owner during native writing.
        crate::labels::write(unsafe { &(*labels).value }, Writer(stream))
    })
    .map_or_else(|status| status, |()| 0)
}

/// Direct native summary output, preserving native validation and write sequence.
/// # Safety
/// Input is a live immutable owner; stream satisfies WriteStream's contract.
/// Partial emitted bytes are retained on failure. No implicit flush or close.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_untied_model_write_summary_stream(
    model: *const ShiroRsUntiedModel,
    stream: *const ShiroRsWriteStream,
) -> u32 {
    for status in [range(model, 1), range(stream, 1)] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live initialized descriptor.
    let stream = unsafe { stream.read() };
    boundary(|| {
        if stream.write.is_none() {
            return Err(io::Error::other("missing write callback"));
        }
        // SAFETY: Live readable owner during native writing.
        unsafe { &(*model).value }.write_summary(Writer(stream))
    })
    .map_or_else(|status| status, |()| 0)
}

/// Direct native buffered index reader without additional read-ahead or buffering.
/// # Safety
/// Stream satisfies BufferedReadStream's borrowed-buffer contract; all input
/// owners are live and immutable. Output is independent aligned writable storage
/// holding no live owner on success. Failure retains output, not consumed input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_index_read_stream(
    stream: *const ShiroRsBufferedReadStream,
    directory: *const ShiroRsPath,
    left: *const ShiroRsStrings,
    right: *const ShiroRsStrings,
    output: *mut *mut ShiroRsIndexEntries,
) -> u32 {
    for status in [
        range(stream, 1),
        range(directory, 1),
        range(left, 1),
        range(right, 1),
    ] {
        if let Err(status) = status {
            return status;
        }
    }
    // SAFETY: Live initialized descriptor.
    let stream = unsafe { stream.read() };
    // SAFETY: Complete live input owners and independent output.
    unsafe {
        result(output, || {
            if stream.fill.is_none() || stream.consume.is_none() {
                return Err(io::Error::other("missing buffered read callback"));
            }
            Ok(Box::into_raw(Box::new(ShiroRsIndexEntries {
                values: crate::index::read(
                    BufferedReader(stream),
                    &(*directory).value,
                    &(*left).values,
                    &(*right).values,
                )?,
            })))
        })
    }
}

/// Explicitly flush a borrowed stream; interrupted status is returned as IO failure.
/// # Safety
/// Stream is live aligned initialized metadata satisfying WriteStream's callback
/// and independent-context contract. No callback/context is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shiro_rs_write_stream_flush(stream: *const ShiroRsWriteStream) -> u32 {
    if let Err(status) = range(stream, 1) {
        return status;
    }
    // SAFETY: Initialized immutable metadata.
    let stream = unsafe { stream.read() };
    boundary(|| Writer(stream).flush()).map_or_else(|status| status, |()| 0)
}
