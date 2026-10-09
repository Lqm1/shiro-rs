use shiro_rs_capi::*;
use std::ptr::{null, null_mut};
unsafe fn owned(values: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable bytes and independent output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(values.as_ptr(), values.len(), &mut output) },
        0
    );
    output
}
unsafe fn copied(owner: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live immutable owner and independent output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(owner, &mut count), 0);
        let mut output = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(owner, 0, output.as_mut_ptr(), count), 0);
        output
    }
}
fn native(text: &str) -> Vec<u8> {
    if cfg!(windows) {
        text.encode_utf16().flat_map(u16::to_le_bytes).collect()
    } else {
        text.as_bytes().to_vec()
    }
}
#[test]
fn every_native_unit_and_suffix_survive_source_parent_and_snapshot_release() {
    let expected = if cfg!(windows) {
        [0x66u16, 0xd800, 0, 0xdc00, 0x2f, 0x6f]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>()
    } else {
        vec![0x66, 0xff, 0x80, 0, 0xfe, 0x2f, 0x6f]
    };
    assert_eq!(
        shiro_rs_path_native_encoding(),
        if cfg!(windows) { 2 } else { 1 }
    );
    // SAFETY: Unique owners and independent output slots throughout.
    unsafe {
        let mut input = owned(&expected);
        let mut path = null_mut();
        let mut clone = null_mut();
        assert_eq!(shiro_rs_path_from_native_bytes(input, &mut path), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        assert_eq!(shiro_rs_path_clone(path, &mut clone), 0);
        assert_eq!(shiro_rs_path_release(&mut path), 0);
        let suffix = ".param\0\u{1f642}";
        let mut suffix_owner = owned(suffix.as_bytes());
        assert_eq!(
            shiro_rs_path_append_suffix(clone, suffix_owner, &mut path),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut suffix_owner), 0);
        let mut original = null_mut();
        let mut appended = null_mut();
        assert_eq!(shiro_rs_path_native_bytes(clone, &mut original), 0);
        assert_eq!(shiro_rs_path_native_bytes(path, &mut appended), 0);
        assert_eq!(shiro_rs_path_release(&mut clone), 0);
        assert_eq!(shiro_rs_path_release(&mut path), 0);
        assert_eq!(copied(original), expected);
        let mut complete = expected;
        complete.extend(native(suffix));
        assert_eq!(copied(appended), complete);
        assert_eq!(shiro_rs_bytes_release(&mut original), 0);
        assert_eq!(shiro_rs_bytes_release(&mut appended), 0);
    }
}
#[test]
fn full_unicode_empty_paths_and_rejected_inputs_preserve_output_owners() {
    // SAFETY: Unique live owners/independent outputs; deliberately invalid nulls
    // are rejected before access.
    unsafe {
        for text in ["", "data/sub/voice\0\u{1f642}", "C:\\folder\\clip.wav"] {
            let mut input = owned(text.as_bytes());
            let mut path = null_mut();
            assert_eq!(shiro_rs_path_from_utf8(input, &mut path), 0);
            assert_eq!(shiro_rs_bytes_release(&mut input), 0);
            let mut output = null_mut();
            assert_eq!(shiro_rs_path_native_bytes(path, &mut output), 0);
            assert_eq!(copied(output), native(text));
            assert_eq!(shiro_rs_bytes_release(&mut output), 0);
            let mut invalid = owned(&[0xff]);
            let mut retained = path;
            assert_eq!(shiro_rs_path_from_utf8(invalid, &mut retained), 3);
            assert_eq!(retained, path);
            assert_eq!(shiro_rs_path_append_suffix(path, invalid, &mut retained), 3);
            assert_eq!(retained, path);
            if cfg!(windows) {
                assert_eq!(shiro_rs_path_from_native_bytes(invalid, &mut retained), 3);
                assert_eq!(retained, path);
            }
            assert_eq!(shiro_rs_path_from_utf8(null(), &mut retained), 1);
            assert_eq!(retained, path);
            assert_eq!(shiro_rs_path_from_native_bytes(null(), &mut retained), 1);
            assert_eq!(retained, path);
            assert_eq!(shiro_rs_path_clone(path, null_mut()), 1);
            assert_eq!(shiro_rs_path_native_bytes(path, null_mut()), 1);
            assert_eq!(shiro_rs_path_release(null_mut()), 1);
            assert_eq!(shiro_rs_bytes_release(&mut invalid), 0);
            assert_eq!(shiro_rs_path_release(&mut path), 0);
            assert_eq!(shiro_rs_path_release(&mut path), 0);
        }
    }
}
