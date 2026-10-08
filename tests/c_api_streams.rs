#![cfg(feature = "c-api")]
use shiro_rs::{c_api::*, labels, untying};
use std::{
    ffi::c_void,
    io::{self, BufRead, Read, Write},
    ptr::{null, null_mut},
};

#[derive(Clone, Debug)]
struct Channel {
    bytes: Vec<u8>,
    position: usize,
    chunk: usize,
    interrupt: bool,
    fail_at: Option<usize>,
    calls: usize,
    flushes: usize,
    trace: Vec<(char, usize)>,
}
impl Channel {
    fn new(bytes: Vec<u8>, chunk: usize) -> Self {
        Self {
            bytes,
            position: 0,
            chunk,
            interrupt: true,
            fail_at: None,
            calls: 0,
            flushes: 0,
            trace: Vec::new(),
        }
    }
    fn step(&mut self, kind: char, amount: usize) -> io::Result<()> {
        self.trace.push((kind, amount));
        self.calls += 1;
        if self.interrupt {
            self.interrupt = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.fail_at.is_some_and(|limit| self.position >= limit) {
            return Err(io::Error::other("injected IO failure"));
        }
        Ok(())
    }
    fn reader(&mut self) -> ShiroRsReadStream {
        ShiroRsReadStream {
            context: std::ptr::from_mut(self).cast(),
            read: Some(read),
        }
    }
    fn writer(&mut self) -> ShiroRsWriteStream {
        ShiroRsWriteStream {
            context: std::ptr::from_mut(self).cast(),
            write: Some(write),
            flush: Some(flush),
        }
    }
    fn buffered(&mut self) -> ShiroRsBufferedReadStream {
        ShiroRsBufferedReadStream {
            context: std::ptr::from_mut(self).cast(),
            fill: Some(fill),
            consume: Some(consume),
        }
    }
}
impl Read for Channel {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.step('r', output.len())?;
        let count = output
            .len()
            .min(self.chunk)
            .min(self.bytes.len() - self.position);
        output[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}
impl Write for Channel {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        self.step('w', input.len())?;
        let count = input.len().min(self.chunk);
        self.bytes.extend_from_slice(&input[..count]);
        self.position += count;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        self.step('s', 0)
    }
}
impl BufRead for Channel {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.step('f', 0)?;
        let end = self.bytes.len().min(self.position + self.chunk);
        Ok(&self.bytes[self.position..end])
    }
    fn consume(&mut self, count: usize) {
        assert!(count <= self.chunk.min(self.bytes.len() - self.position));
        self.trace.push(('c', count));
        self.position += count;
    }
}
fn status(value: io::Result<usize>, output: *mut usize) -> u32 {
    match value {
        Ok(count) => {
            // SAFETY: Library provides independent initialized writable count storage.
            unsafe { output.write(count) };
            SHIRO_RS_IO_SUCCESS
        }
        Err(e) if e.kind() == io::ErrorKind::Interrupted => SHIRO_RS_IO_INTERRUPTED,
        Err(_) => SHIRO_RS_IO_ERROR,
    }
}
unsafe extern "C" fn excessive_write(
    _context: *mut c_void,
    _buffer: *const u8,
    capacity: usize,
    count: *mut usize,
) -> u32 {
    // SAFETY: Valid count slot; intentionally report an invalid transfer length.
    unsafe { count.write(capacity + 1) };
    0
}
unsafe extern "C" fn invalid_fill(
    context: *mut c_void,
    buffer: *mut *const u8,
    count: *mut usize,
) -> u32 {
    // SAFETY: Valid independent pointer/count slots; no referenced bytes accessed.
    unsafe {
        buffer.write(null());
        count.write(if context.is_null() { 1 } else { usize::MAX });
    }
    0
}
static INVALID_CONSUMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
unsafe extern "C" fn never_consume(_context: *mut c_void, _count: usize) {
    INVALID_CONSUMED.store(true, std::sync::atomic::Ordering::Relaxed);
}
unsafe extern "C" fn read(
    context: *mut c_void,
    buffer: *mut u8,
    capacity: usize,
    count: *mut usize,
) -> u32 {
    // SAFETY: Unique Channel context and initialized writable library buffer.
    unsafe {
        status(
            (&mut *context.cast::<Channel>())
                .read(std::slice::from_raw_parts_mut(buffer, capacity)),
            count,
        )
    }
}
unsafe extern "C" fn write(
    context: *mut c_void,
    buffer: *const u8,
    capacity: usize,
    count: *mut usize,
) -> u32 {
    // SAFETY: Unique Channel context and initialized readable library buffer.
    unsafe {
        status(
            (&mut *context.cast::<Channel>()).write(std::slice::from_raw_parts(buffer, capacity)),
            count,
        )
    }
}
unsafe extern "C" fn flush(context: *mut c_void) -> u32 {
    // SAFETY: Unique live Channel context for this call.
    match unsafe { (&mut *context.cast::<Channel>()).flush() } {
        Ok(()) => 0,
        Err(e) if e.kind() == io::ErrorKind::Interrupted => 1,
        Err(_) => 2,
    }
}
unsafe extern "C" fn fill(context: *mut c_void, buffer: *mut *const u8, count: *mut usize) -> u32 {
    // SAFETY: Unique live context; returned storage remains live until consume/fill.
    match unsafe { (&mut *context.cast::<Channel>()).fill_buf() } {
        Ok(bytes) => {
            // SAFETY: Library supplies independent writable pointer/count slots.
            unsafe {
                buffer.write(bytes.as_ptr());
                count.write(bytes.len());
            }
            0
        }
        Err(e) if e.kind() == io::ErrorKind::Interrupted => 1,
        Err(_) => 2,
    }
}
unsafe extern "C" fn consume(context: *mut c_void, count: usize) {
    // SAFETY: Unique context and native count within the previous returned buffer.
    unsafe { (&mut *context.cast::<Channel>()).consume(count) };
}
unsafe extern "C" fn excessive(
    _context: *mut c_void,
    _buffer: *mut u8,
    capacity: usize,
    count: *mut usize,
) -> u32 {
    // SAFETY: Writable library count storage; deliberate invalid reported count.
    unsafe { count.write(capacity + 1) };
    0
}
unsafe fn owned(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable bytes and independent output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn bytes(owner: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live owner and independent initialized output storage.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(owner, &mut count), 0);
        let mut output = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(owner, 0, output.as_mut_ptr(), count), 0);
        output
    }
}
unsafe fn floats(owner: *const ShiroRsArrayF32) -> Vec<f32> {
    // SAFETY: Live owner and independent initialized output storage.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_array_f32_length(owner, &mut count), 0);
        let mut output = vec![0.0; count];
        assert_eq!(
            shiro_rs_array_f32_copy(owner, 0, output.as_mut_ptr(), count),
            0
        );
        output
    }
}
fn same_trace(actual: &Channel, expected: &Channel) {
    assert_eq!(actual.trace, expected.trace);
    assert_eq!(actual.position, expected.position);
    assert_eq!(actual.bytes, expected.bytes);
    assert_eq!(actual.flushes, expected.flushes);
}

#[test]
fn rawfloat_and_multistream_observation_preserve_native_partial_io_and_bits() {
    let values: Vec<_> = (0..5003)
        .map(|i| f32::from_bits([0, 0x80000000, 1, 0x7fc12345, 0x7f800000, 0xff800000][i % 6]))
        .collect();
    let wire: Vec<_> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    for chunk in [1, 7, 16384] {
        let mut actual = Channel::new(wire.clone(), chunk);
        let mut expected = actual.clone();
        let native = shiro_rs::rawfloat::read(&mut expected, values.len()).unwrap();
        // SAFETY: Independent Channel contexts/owners/output storage.
        unsafe {
            let mut array = null_mut();
            assert_eq!(
                shiro_rs_rawfloat_read_stream(&actual.reader(), values.len(), &mut array),
                0
            );
            assert_eq!(
                floats(array)
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                native.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
            );
            same_trace(&actual, &expected);
            let mut output = Channel::new(Vec::new(), chunk);
            let mut reference = output.clone();
            shiro_rs::rawfloat::write(&mut reference, &values).unwrap();
            assert_eq!(shiro_rs_rawfloat_write_stream(array, &output.writer()), 0);
            same_trace(&output, &reference);
            assert_eq!(output.bytes, wire);
            assert_eq!(output.flushes, 0);
            assert_eq!(shiro_rs_write_stream_flush(&output.writer()), 0);
            assert_eq!(output.flushes, 1);
            assert_eq!(shiro_rs_array_f32_release(&mut array), 0);
        }
    }
    let data = include_bytes!("fixtures/init-input.bin");
    let dimensions = [2usize, 1];
    for chunk in [1, 13, 16384] {
        let mut actual = Channel::new(data.to_vec(), chunk);
        let mut expected = actual.clone();
        let native = shiro_rs::dataset::read_observation(&mut expected, &dimensions, 1000).unwrap();
        // SAFETY: Independent IO context, readable dimensions and output owners.
        unsafe {
            let mut observation = null_mut();
            let mut wire = null_mut();
            assert_eq!(
                shiro_rs_observation_read_stream(
                    &actual.reader(),
                    dimensions.as_ptr(),
                    2,
                    1000,
                    &mut observation
                ),
                0
            );
            assert_eq!(shiro_rs_observation_write_bytes(observation, &mut wire), 0);
            let mut reference = Vec::new();
            native.write_to(&mut reference).unwrap();
            assert_eq!(bytes(wire), reference);
            same_trace(&actual, &expected);
            assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
            assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        }
    }
}

#[test]
fn rawfloat_errors_keep_output_but_preserve_native_consumption_and_partial_writes() {
    // SAFETY: Independent owners and contexts; invalid descriptors are rejected.
    unsafe {
        let sample = [f32::from_bits(0x7fc12345), -0.0, 1.0];
        let mut array = null_mut();
        assert_eq!(shiro_rs_array_f32_create(sample.as_ptr(), 3, &mut array), 0);
        for (data, budget, failure) in [
            (vec![0; 5], 8, None),
            (vec![0; 12], 1, None),
            (vec![0; 12], 8, Some(5)),
        ] {
            let mut actual = Channel::new(data, 1);
            actual.fail_at = failure;
            let mut expected = actual.clone();
            assert!(shiro_rs::rawfloat::read(&mut expected, budget).is_err());
            let mut retained = array;
            assert_eq!(
                shiro_rs_rawfloat_read_stream(&actual.reader(), budget, &mut retained),
                3
            );
            assert_eq!(retained, array);
            same_trace(&actual, &expected);
        }
        let invalid = ShiroRsReadStream {
            context: null_mut(),
            read: Some(excessive),
        };
        let mut retained = array;
        assert_eq!(
            shiro_rs_rawfloat_read_stream(&invalid, 99, &mut retained),
            3
        );
        assert_eq!(retained, array);
        let missing = ShiroRsReadStream {
            context: null_mut(),
            read: None,
        };
        assert_eq!(
            shiro_rs_rawfloat_read_stream(&missing, 99, &mut retained),
            3
        );
        assert_eq!(shiro_rs_rawfloat_read_stream(null(), 99, &mut retained), 1);
        let mut untouched = Channel::new(vec![0; 4], 1);
        assert_eq!(
            shiro_rs_rawfloat_read_stream(&untouched.reader(), 9, null_mut()),
            1
        );
        assert_eq!(untouched.calls, 0);
        let mut observation = null_mut();
        assert_eq!(
            shiro_rs_observation_read_stream(&untouched.reader(), null(), 0, 9, &mut observation),
            3
        );
        assert_eq!(untouched.calls, 0);
        assert_eq!(
            shiro_rs_observation_read_stream(&untouched.reader(), null(), 1, 9, &mut observation),
            1
        );
        for (chunk, failure) in [(0, None), (1, Some(5))] {
            let mut actual = Channel::new(Vec::new(), chunk);
            actual.fail_at = failure;
            let mut expected = actual.clone();
            assert!(shiro_rs::rawfloat::write(&mut expected, &sample).is_err());
            assert_eq!(shiro_rs_rawfloat_write_stream(array, &actual.writer()), 3);
            same_trace(&actual, &expected);
        }
        let mut output = Channel::new(Vec::new(), 1);
        assert_eq!(shiro_rs_write_stream_flush(&output.writer()), 3);
        assert_eq!(output.flushes, 1);
        assert_eq!(shiro_rs_write_stream_flush(&output.writer()), 0);
        assert_eq!(output.flushes, 2);
        let missing_writer = ShiroRsWriteStream {
            context: null_mut(),
            write: None,
            flush: None,
        };
        assert_eq!(shiro_rs_rawfloat_write_stream(array, &missing_writer), 3);
        assert_eq!(shiro_rs_write_stream_flush(&missing_writer), 3);
        assert_eq!(shiro_rs_write_stream_flush(null()), 1);
        assert_eq!(shiro_rs_array_f32_release(&mut array), 0);
    }
}

#[test]
fn buffered_index_matches_native_fields_fill_consume_and_errors_without_read_ahead() {
    let directory = std::path::Path::new("data");
    let original = include_bytes!("fixtures/index-original.txt").as_slice();
    // SAFETY: Independent full owners and callback contexts.
    unsafe {
        let mut directory_bytes = owned(b"data");
        let mut path = null_mut();
        let mut padding = null_mut();
        assert_eq!(shiro_rs_path_from_utf8(directory_bytes, &mut path), 0);
        assert_eq!(shiro_rs_bytes_release(&mut directory_bytes), 0);
        assert_eq!(shiro_rs_strings_create(null(), 0, &mut padding), 0);
        for data in [
            original,
            b"\r\nclip, aa  bb \r\n\nsilent,\n",
            b"",
            b"\n\r\nbad\n",
            b"ok,a\n\nclip,a,b\n",
            b"clip,\xff\n",
        ] {
            for chunk in [1, 7, 999] {
                let mut actual = Channel::new(data.to_vec(), chunk);
                let mut expected = actual.clone();
                let native = shiro_rs::index::read(&mut expected, directory, &[], &[]);
                let mut entries = null_mut();
                assert_eq!(
                    shiro_rs_index_read_stream(
                        &actual.buffered(),
                        path,
                        padding,
                        padding,
                        &mut entries
                    ),
                    if native.is_ok() { 0 } else { 3 }
                );
                same_trace(&actual, &expected);
                if let Ok(reference) = native {
                    let mut length = 0;
                    assert_eq!(shiro_rs_index_entries_length(entries, &mut length), 0);
                    assert_eq!(length, reference.len());
                    for (i, entry) in reference.iter().enumerate() {
                        let mut stem = null_mut();
                        let mut phones = null_mut();
                        let mut wire = null_mut();
                        assert_eq!(shiro_rs_index_entries_get_stem(entries, i, &mut stem), 0);
                        assert_eq!(shiro_rs_path_native_bytes(stem, &mut wire), 0);
                        #[cfg(windows)]
                        let native_units: Vec<_> = {
                            use std::os::windows::ffi::OsStrExt;
                            entry
                                .stem
                                .as_os_str()
                                .encode_wide()
                                .flat_map(u16::to_le_bytes)
                                .collect()
                        };
                        #[cfg(unix)]
                        let native_units = {
                            use std::os::unix::ffi::OsStrExt;
                            entry.stem.as_os_str().as_bytes().to_vec()
                        };
                        assert_eq!(bytes(wire), native_units);
                        assert_eq!(
                            shiro_rs_index_entries_get_phonemes(entries, i, &mut phones),
                            0
                        );
                        assert_eq!(shiro_rs_strings_length(phones, &mut length), 0);
                        assert_eq!(length, entry.phonemes.len());
                        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
                        for (j, expected) in entry.phonemes.iter().enumerate() {
                            assert_eq!(shiro_rs_strings_get(phones, j, &mut wire), 0);
                            assert_eq!(bytes(wire), expected.as_bytes());
                            assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
                        }
                        assert_eq!(shiro_rs_path_release(&mut stem), 0);
                        assert_eq!(shiro_rs_strings_release(&mut phones), 0);
                    }
                    assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
                } else {
                    assert!(entries.is_null());
                }
            }
        }
        let mut actual = Channel::new(original.to_vec(), 1);
        actual.fail_at = Some(9);
        let mut expected = actual.clone();
        assert!(shiro_rs::index::read(&mut expected, directory, &[], &[]).is_err());
        let mut entries = null_mut();
        assert_eq!(
            shiro_rs_index_read_stream(&actual.buffered(), path, padding, padding, &mut entries),
            3
        );
        same_trace(&actual, &expected);
        let missing = ShiroRsBufferedReadStream {
            context: null_mut(),
            fill: None,
            consume: None,
        };
        assert_eq!(
            shiro_rs_index_read_stream(&missing, path, padding, padding, &mut entries),
            3
        );
        assert_eq!(
            shiro_rs_index_read_stream(null(), path, padding, padding, &mut entries),
            1
        );
        assert_eq!(shiro_rs_path_release(&mut path), 0);
        assert_eq!(shiro_rs_strings_release(&mut padding), 0);
    }
}

#[test]
fn label_and_summary_streams_preserve_native_format_trace_and_late_failures() {
    let native_labels = labels::parse(include_str!("fixtures/labels-input.txt")).unwrap();
    let native_model =
        shiro_rs::hsmm::Model::read_from(include_bytes!("fixtures/init-c-aligned.hsmm").as_slice())
            .unwrap();
    let document: labels::SegmentationDocument =
        serde_json::from_slice(include_bytes!("fixtures/align-c-isolated.json")).unwrap();
    let native_untied = untying::untie(&native_model, &document).unwrap();
    // SAFETY: Live independent owners and callback contexts throughout.
    unsafe {
        let mut input = owned(include_bytes!("fixtures/labels-input.txt"));
        let mut labels_owner = null_mut();
        assert_eq!(shiro_rs_labels_parse(input, &mut labels_owner), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        input = owned(include_bytes!("fixtures/init-c-aligned.hsmm"));
        let mut model = null_mut();
        assert_eq!(
            shiro_rs_model_read_bytes(input, 16 * 1024 * 1024, &mut model),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        input = owned(include_bytes!("fixtures/align-c-isolated.json"));
        let mut untied = null_mut();
        assert_eq!(shiro_rs_untie(model, input, &mut untied), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        for chunk in [1, 13, 16384] {
            for failure in [None, Some(7)] {
                let mut actual = Channel::new(Vec::new(), chunk);
                actual.fail_at = failure;
                let mut expected = actual.clone();
                let result = labels::write(&native_labels, &mut expected);
                assert_eq!(
                    shiro_rs_labels_write_stream(labels_owner, &actual.writer()),
                    if result.is_ok() { 0 } else { 3 }
                );
                same_trace(&actual, &expected);
                let mut actual = Channel::new(Vec::new(), chunk);
                actual.fail_at = failure;
                let mut expected = actual.clone();
                let result = native_untied.write_summary(&mut expected);
                assert_eq!(
                    shiro_rs_untied_model_write_summary_stream(untied, &actual.writer()),
                    if result.is_ok() { 0 } else { 3 }
                );
                same_trace(&actual, &expected);
                if result.is_ok() {
                    assert_eq!(actual.bytes, include_bytes!("fixtures/untie-c-summary.txt"));
                }
            }
        }
        let mut first = owned(b"valid");
        let mut bad = owned(b"bad\tname");
        let rows = [
            ShiroRsLabelInput {
                start: 0.0,
                end: 1.0,
                name: first,
            },
            ShiroRsLabelInput {
                start: 1.0,
                end: 2.0,
                name: bad,
            },
        ];
        let mut invalid = null_mut();
        assert_eq!(shiro_rs_labels_create(rows.as_ptr(), 2, &mut invalid), 0);
        let native = [
            labels::Label {
                start: 0.0,
                end: 1.0,
                name: "valid".into(),
            },
            labels::Label {
                start: 1.0,
                end: 2.0,
                name: "bad\tname".into(),
            },
        ];
        let mut actual = Channel::new(Vec::new(), 1);
        let mut expected = actual.clone();
        assert!(labels::write(&native, &mut expected).is_err());
        assert_eq!(shiro_rs_labels_write_stream(invalid, &actual.writer()), 3);
        same_trace(&actual, &expected);
        assert!(!actual.bytes.is_empty());
        assert_eq!(shiro_rs_labels_write_stream(null(), &actual.writer()), 1);
        assert_eq!(
            shiro_rs_untied_model_write_summary_stream(untied, null()),
            1
        );
        assert_eq!(shiro_rs_bytes_release(&mut first), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bad), 0);
        assert_eq!(shiro_rs_labels_release(&mut invalid), 0);
        assert_eq!(shiro_rs_labels_release(&mut labels_owner), 0);
        assert_eq!(shiro_rs_untied_model_release(&mut untied), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}

#[test]
fn invalid_callback_counts_borrowed_ranges_and_late_summary_fields_are_rejected() {
    // SAFETY: Independent initialized outputs and unique live owners. Deliberate
    // invalid counts/buffer pointers are rejected before dereference or consume.
    unsafe {
        let mut array = null_mut();
        assert_eq!(shiro_rs_array_f32_create([1.0].as_ptr(), 1, &mut array), 0);
        let writer = ShiroRsWriteStream {
            context: null_mut(),
            write: Some(excessive_write),
            flush: None,
        };
        assert_eq!(shiro_rs_rawfloat_write_stream(array, &writer), 3);
        let mut directory_bytes = owned(b"data");
        let mut directory = null_mut();
        let mut padding = null_mut();
        let mut entries = null_mut();
        assert_eq!(shiro_rs_path_from_utf8(directory_bytes, &mut directory), 0);
        assert_eq!(shiro_rs_bytes_release(&mut directory_bytes), 0);
        assert_eq!(shiro_rs_strings_create(null(), 0, &mut padding), 0);
        assert_eq!(shiro_rs_index_entries_create(null(), 0, &mut entries), 0);
        for context in [null_mut(), std::ptr::dangling_mut::<u8>().cast()] {
            INVALID_CONSUMED.store(false, std::sync::atomic::Ordering::Relaxed);
            let reader = ShiroRsBufferedReadStream {
                context,
                fill: Some(invalid_fill),
                consume: Some(never_consume),
            };
            let mut retained = entries;
            assert_eq!(
                shiro_rs_index_read_stream(&reader, directory, padding, padding, &mut retained),
                3
            );
            assert_eq!(retained, entries);
            assert!(!INVALID_CONSUMED.load(std::sync::atomic::Ordering::Relaxed));
        }
        let mut untouched = Channel::new(b"clip,a\n".to_vec(), 1);
        assert_eq!(
            shiro_rs_index_read_stream(
                &untouched.buffered(),
                directory,
                padding,
                padding,
                null_mut()
            ),
            1
        );
        assert_eq!(untouched.calls, 0);
        let mut input = owned(include_bytes!("fixtures/init-c-aligned.hsmm"));
        let mut model = null_mut();
        assert_eq!(
            shiro_rs_model_read_bytes(input, 16 * 1024 * 1024, &mut model),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        input = owned(include_bytes!("fixtures/align-c-isolated.json"));
        let assignments = [
            ShiroRsAssignment {
                state: 0,
                file: 0,
                segment: 0,
            },
            ShiroRsAssignment {
                state: usize::MAX,
                file: usize::MAX,
                segment: usize::MAX,
            },
        ];
        let mut untied = null_mut();
        assert_eq!(
            shiro_rs_untied_model_create(model, input, assignments.as_ptr(), 2, &mut untied),
            0
        );
        let mut output = Channel::new(Vec::new(), 1);
        assert_eq!(
            shiro_rs_untied_model_write_summary_stream(untied, &output.writer()),
            3
        );
        assert_eq!(output.calls, 0);
        assert!(output.bytes.is_empty());
        assert_eq!(shiro_rs_untied_model_release(&mut untied), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
        assert_eq!(shiro_rs_strings_release(&mut padding), 0);
        assert_eq!(shiro_rs_path_release(&mut directory), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut array), 0);
    }
}
