use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

unsafe fn bytes(value: &[u8]) -> *mut ShiroRsBytes {
    let mut owner = null_mut();
    // SAFETY: Readable bytes and independent output slot.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(value.as_ptr(), value.len(), &mut owner) },
        0
    );
    owner
}
unsafe fn byte_values(owner: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live readable owner and independent output storage.
    unsafe {
        let mut length = 0;
        assert_eq!(shiro_rs_bytes_length(owner, &mut length), 0);
        let mut values = vec![0; length];
        assert_eq!(
            shiro_rs_bytes_copy(owner, 0, values.as_mut_ptr(), length),
            0
        );
        values
    }
}
unsafe fn strings(values: &[&str]) -> *mut ShiroRsStrings {
    // SAFETY: Unique temporary byte owners and independent output slot.
    unsafe {
        let mut owners: Vec<_> = values.iter().map(|s| bytes(s.as_bytes())).collect();
        let pointers: Vec<_> = owners.iter().map(|&p| p.cast_const()).collect();
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_strings_create(pointers.as_ptr(), pointers.len(), &mut output),
            0
        );
        for owner in &mut owners {
            assert_eq!(shiro_rs_bytes_release(owner), 0);
        }
        output
    }
}
unsafe fn string_values(owner: *const ShiroRsStrings) -> Vec<String> {
    // SAFETY: Live immutable owner; snapshots uniquely released after reading.
    unsafe {
        let mut length = 0;
        assert_eq!(shiro_rs_strings_length(owner, &mut length), 0);
        (0..length)
            .map(|i| {
                let mut snapshot = null_mut();
                assert_eq!(shiro_rs_strings_get(owner, i, &mut snapshot), 0);
                let value = String::from_utf8(byte_values(snapshot)).unwrap();
                assert_eq!(shiro_rs_bytes_release(&mut snapshot), 0);
                value
            })
            .collect()
    }
}
unsafe fn path(text: &str) -> *mut ShiroRsPath {
    // SAFETY: Temporary byte owner and independent output slot.
    unsafe {
        let mut input = bytes(text.as_bytes());
        let mut output = null_mut();
        assert_eq!(shiro_rs_path_from_utf8(input, &mut output), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        output
    }
}
unsafe fn path_values(owner: *const ShiroRsPath) -> Vec<u8> {
    // SAFETY: Live immutable path and unique snapshot ownership.
    unsafe {
        let mut snapshot = null_mut();
        assert_eq!(shiro_rs_path_native_bytes(owner, &mut snapshot), 0);
        let value = byte_values(snapshot);
        assert_eq!(shiro_rs_bytes_release(&mut snapshot), 0);
        value
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
fn original_index_all_fields_and_native_padding_and_tokenization() {
    let reference: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/index-original.json"
    ))
    .unwrap();
    // SAFETY: All inputs uniquely owned and output snapshots independent.
    unsafe {
        let mut directory = path("data");
        let mut left = strings(&["left"]);
        let mut right = strings(&["right"]);
        let mut input = bytes(include_bytes!("../../../tests/fixtures/index-original.txt"));
        let mut entries = null_mut();
        assert_eq!(
            shiro_rs_index_read_bytes(input, directory, left, right, &mut entries),
            0
        );
        let mut length = 0;
        assert_eq!(shiro_rs_index_entries_length(entries, &mut length), 0);
        assert_eq!(length, 3);
        for (i, expected) in reference.as_array().unwrap().iter().enumerate() {
            let mut stem = null_mut();
            let mut phones = null_mut();
            assert_eq!(shiro_rs_index_entries_get_stem(entries, i, &mut stem), 0);
            assert_eq!(
                shiro_rs_index_entries_get_phonemes(entries, i, &mut phones),
                0
            );
            let expected_path = std::path::Path::new("data").join(
                expected["path"]
                    .as_str()
                    .unwrap()
                    .strip_prefix("data/")
                    .unwrap(),
            );
            assert_eq!(path_values(stem), native(expected_path.to_str().unwrap()));
            assert_eq!(
                serde_json::to_value(string_values(phones)).unwrap(),
                expected["phonemes"]
            );
            assert_eq!(shiro_rs_path_release(&mut stem), 0);
            assert_eq!(shiro_rs_strings_release(&mut phones), 0);
        }
        assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        assert_eq!(shiro_rs_strings_release(&mut left), 0);
        assert_eq!(shiro_rs_strings_release(&mut right), 0);
        left = strings(&["", "pad\0\u{1f642}"]);
        right = strings(&[" right "]);
        for data in [b"\r\nclip, aa  bb \r\n\nsilent,\n".as_slice(), b""] {
            input = bytes(data);
            assert_eq!(
                shiro_rs_index_read_bytes(input, directory, left, right, &mut entries),
                0
            );
            let independent = shiro_rs::index::read(
                std::io::Cursor::new(data),
                std::path::Path::new("data"),
                &["".into(), "pad\0\u{1f642}".into()],
                &[" right ".into()],
            )
            .unwrap();
            assert_eq!(shiro_rs_index_entries_length(entries, &mut length), 0);
            assert_eq!(length, independent.len());
            for (i, expected) in independent.iter().enumerate() {
                let mut phones = null_mut();
                assert_eq!(
                    shiro_rs_index_entries_get_phonemes(entries, i, &mut phones),
                    0
                );
                assert_eq!(string_values(phones), expected.phonemes);
                assert_eq!(shiro_rs_strings_release(&mut phones), 0);
            }
            assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
            assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        }
        assert_eq!(shiro_rs_path_release(&mut directory), 0);
        assert_eq!(shiro_rs_strings_release(&mut left), 0);
        assert_eq!(shiro_rs_strings_release(&mut right), 0);
    }
}

#[test]
fn arbitrary_fields_repeated_owners_nonunicode_and_independent_lifetimes() {
    let native_units = if cfg!(windows) {
        [0x66u16, 0xd800, 0, 0xdc00]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>()
    } else {
        vec![0x66, 0xff, 0, 0x80]
    };
    // SAFETY: Unique live owners, repeated immutable inputs and independent outputs.
    unsafe {
        let mut wire = bytes(&native_units);
        let mut stem = null_mut();
        assert_eq!(shiro_rs_path_from_native_bytes(wire, &mut stem), 0);
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
        let mut text = bytes(b"repeat\0 ");
        let mut empty = bytes(b"");
        let pointers = [text.cast_const(), empty.cast_const(), text.cast_const()];
        let mut phones = null_mut();
        assert_eq!(
            shiro_rs_strings_create(pointers.as_ptr(), 3, &mut phones),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut text), 0);
        assert_eq!(shiro_rs_bytes_release(&mut empty), 0);
        let mut phone_clone = null_mut();
        assert_eq!(shiro_rs_strings_clone(phones, &mut phone_clone), 0);
        let entry = ShiroRsIndexEntryInput {
            stem,
            phonemes: phones,
        };
        let mut entries = null_mut();
        assert_eq!(
            shiro_rs_index_entries_create([entry, entry].as_ptr(), 2, &mut entries),
            0
        );
        assert_eq!(shiro_rs_path_release(&mut stem), 0);
        assert_eq!(shiro_rs_strings_release(&mut phones), 0);
        let mut clone = null_mut();
        assert_eq!(shiro_rs_index_entries_clone(entries, &mut clone), 0);
        assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
        let mut length = 0;
        assert_eq!(shiro_rs_index_entries_length(clone, &mut length), 0);
        assert_eq!(length, 2);
        assert_eq!(shiro_rs_index_entries_get_stem(clone, 1, &mut stem), 0);
        assert_eq!(
            shiro_rs_index_entries_get_phonemes(clone, 1, &mut phones),
            0
        );
        assert_eq!(shiro_rs_index_entries_release(&mut clone), 0);
        assert_eq!(path_values(stem), native_units);
        assert_eq!(string_values(phones), ["repeat\0 ", "", "repeat\0 "]);
        assert_eq!(string_values(phone_clone), string_values(phones));
        // A non-Unicode directory retains every unit after the native join.
        let mut input = bytes(b"child,\n");
        let mut no_padding = strings(&[]);
        assert_eq!(
            shiro_rs_index_read_bytes(input, stem, no_padding, no_padding, &mut entries),
            0
        );
        let mut joined = null_mut();
        assert_eq!(shiro_rs_index_entries_get_stem(entries, 0, &mut joined), 0);
        let mut expected = native_units;
        expected.extend(native(if cfg!(windows) { "\\child" } else { "/child" }));
        assert_eq!(path_values(joined), expected);
        for owner in [&mut stem, &mut joined] {
            assert_eq!(shiro_rs_path_release(owner), 0);
        }
        for owner in [&mut phones, &mut phone_clone, &mut no_padding] {
            assert_eq!(shiro_rs_strings_release(owner), 0);
        }
        assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        assert_eq!(shiro_rs_index_entries_create(null(), 0, &mut entries), 0);
        assert_eq!(shiro_rs_index_entries_length(entries, &mut length), 0);
        assert_eq!(length, 0);
        assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
    }
}

#[test]
fn late_invalid_inputs_and_ranges_retain_all_output_owners() {
    // SAFETY: Invalid null, alignment and excessive ranges are rejected before
    // dereference; sentinel output owners remain live until uniquely released.
    unsafe {
        let mut phones = strings(&["sentinel"]);
        let original_phones = phones;
        let mut valid = bytes(b"valid");
        let mut invalid = bytes(&[0xff]);
        let values = [valid.cast_const(), invalid.cast_const()];
        assert_eq!(shiro_rs_strings_create(values.as_ptr(), 2, &mut phones), 3);
        assert_eq!(phones, original_phones);
        assert_eq!(shiro_rs_strings_create(null(), 1, &mut phones), 1);
        assert_eq!(shiro_rs_strings_create(null(), usize::MAX, &mut phones), 2);
        assert_eq!(
            shiro_rs_strings_create([null()].as_ptr(), 1, &mut phones),
            1
        );
        assert_eq!(shiro_rs_strings_create(values.as_ptr(), 1, null_mut()), 1);
        assert_eq!(shiro_rs_strings_get(phones, usize::MAX, &mut valid), 2);
        assert_eq!(byte_values(valid), b"valid");
        let mut directory = path("data");
        let entry = ShiroRsIndexEntryInput {
            stem: directory,
            phonemes: phones,
        };
        let mut entries = null_mut();
        assert_eq!(shiro_rs_index_entries_create(&entry, 1, &mut entries), 0);
        let original_entries = entries;
        let bad = ShiroRsIndexEntryInput {
            stem: null(),
            phonemes: phones,
        };
        assert_eq!(
            shiro_rs_index_entries_create([entry, bad].as_ptr(), 2, &mut entries),
            1
        );
        assert_eq!(
            shiro_rs_index_entries_create(null(), usize::MAX, &mut entries),
            2
        );
        assert_eq!(
            shiro_rs_index_entries_create(std::ptr::dangling::<u8>().cast(), 1, &mut entries),
            1
        );
        assert_eq!(
            shiro_rs_index_entries_get_stem(entries, 1, &mut directory),
            2
        );
        assert_eq!(
            shiro_rs_index_entries_get_phonemes(entries, 1, &mut phones),
            2
        );
        for data in [
            b"\n\r\nbad\n".as_slice(),
            b"ok,a\n\n,b\n",
            b"ok,a\n\nclip,a,b\n",
            b"clip,\xff\n",
        ] {
            let mut input = bytes(data);
            assert_eq!(
                shiro_rs_index_read_bytes(input, directory, phones, phones, &mut entries),
                3
            );
            assert_eq!(entries, original_entries);
            assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        }
        let mut length = 17;
        assert_eq!(shiro_rs_index_entries_length(null(), &mut length), 1);
        assert_eq!(length, 17);
        assert_eq!(shiro_rs_strings_length(null(), &mut length), 1);
        assert_eq!(length, 17);
        assert_eq!(phones, original_phones);
        assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
        assert_eq!(shiro_rs_index_entries_release(&mut entries), 0);
        assert_eq!(shiro_rs_index_entries_release(null_mut()), 1);
        assert_eq!(shiro_rs_strings_release(&mut phones), 0);
        assert_eq!(shiro_rs_strings_release(&mut phones), 0);
        assert_eq!(shiro_rs_strings_release(null_mut()), 1);
        assert_eq!(shiro_rs_path_release(&mut directory), 0);
        for owner in [&mut valid, &mut invalid] {
            assert_eq!(shiro_rs_bytes_release(owner), 0);
        }
    }
}
