use shiro_rs::labels;
use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

unsafe fn owned(values: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable input and independent initialized output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(values.as_ptr(), values.len(), &mut output) },
        0
    );
    output
}
unsafe fn bytes(owner: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live immutable owner and independent outputs.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(owner, &mut count), 0);
        let mut values = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(owner, 0, values.as_mut_ptr(), count), 0);
        values
    }
}
unsafe fn rows(owner: *const ShiroRsLabels) -> Vec<labels::Label> {
    // SAFETY: Live immutable owner and independent descriptor/name outputs.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_labels_length(owner, &mut count), 0);
        let mut rows = Vec::new();
        for index in 0..count {
            let mut info = ShiroRsLabelInfo {
                start: 0.0,
                end: 0.0,
            };
            let mut name = null_mut();
            assert_eq!(shiro_rs_labels_get_info(owner, index, &mut info), 0);
            assert_eq!(shiro_rs_labels_get_name(owner, index, &mut name), 0);
            rows.push(labels::Label {
                start: info.start,
                end: info.end,
                name: String::from_utf8(bytes(name)).unwrap(),
            });
            assert_eq!(shiro_rs_bytes_release(&mut name), 0);
        }
        rows
    }
}
#[test]
fn original_lua_conversion_complete_rows_and_crlf_match_native() {
    let native = labels::parse(include_str!("../../../tests/fixtures/labels-input.txt")).unwrap();
    let map: labels::PhoneMap =
        serde_json::from_slice(include_bytes!("../../../tests/fixtures/labels-phones.json"))
            .unwrap();
    let expected_states = labels::to_states(&native, &map, 0.01).unwrap();
    let original: labels::SegmentationDocument = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/labels-original-seg.json"
    ))
    .unwrap();
    assert_eq!(
        serde_json::to_value(&expected_states).unwrap(),
        serde_json::to_value(&original.files[0].states).unwrap()
    );
    // SAFETY: Unique owners, immutable inputs and independent output slots.
    unsafe {
        let mut text = owned(include_bytes!("../../../tests/fixtures/labels-input.txt"));
        let mut parsed = null_mut();
        assert_eq!(shiro_rs_labels_parse(text, &mut parsed), 0);
        assert_eq!(shiro_rs_bytes_release(&mut text), 0);
        assert_eq!(rows(parsed), native);
        let mut cloned = null_mut();
        assert_eq!(shiro_rs_labels_clone(parsed, &mut cloned), 0);
        assert_eq!(shiro_rs_labels_release(&mut parsed), 0);
        let mut map_json = owned(include_bytes!("../../../tests/fixtures/labels-phones.json"));
        let mut map_owner = null_mut();
        assert_eq!(shiro_rs_phone_map_read_json(map_json, &mut map_owner), 0);
        assert_eq!(shiro_rs_bytes_release(&mut map_json), 0);
        let mut states = null_mut();
        assert_eq!(
            shiro_rs_labels_to_states(cloned, map_owner, 0.01, &mut states),
            0
        );
        assert_eq!(shiro_rs_labels_release(&mut cloned), 0);
        assert_eq!(shiro_rs_phone_map_release(&mut map_owner), 0);
        let mut wire = null_mut();
        assert_eq!(shiro_rs_states_write_json(states, &mut wire), 0);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes(wire)).unwrap(),
            serde_json::to_value(&expected_states).unwrap()
        );
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
        for (include, fixture) in [
            (
                0,
                include_str!("../../../tests/fixtures/labels-original-phones.txt"),
            ),
            (
                1,
                include_str!("../../../tests/fixtures/labels-original-states.txt"),
            ),
        ] {
            let mut converted = null_mut();
            assert_eq!(
                shiro_rs_labels_from_states(states, 0.01, include, &mut converted),
                0
            );
            let expected = labels::from_states(&expected_states, 0.01, include == 1).unwrap();
            assert_eq!(rows(converted), expected);
            let original = labels::parse(fixture).unwrap();
            for (actual, expected) in rows(converted).iter().zip(&original) {
                assert_eq!(actual.name, expected.name);
                assert!((actual.start - expected.start).abs() < 1e-14);
                assert!((actual.end - expected.end).abs() < 1e-14);
            }
            assert_eq!(rows(converted).len(), original.len());
            assert_eq!(shiro_rs_labels_write_bytes(converted, &mut wire), 0);
            let mut native_wire = Vec::new();
            labels::write(&expected, &mut native_wire).unwrap();
            assert_eq!(bytes(wire), native_wire);
            assert_eq!(shiro_rs_labels_release(&mut converted), 0);
            assert_eq!(shiro_rs_labels_parse(wire, &mut converted), 0);
            assert_eq!(rows(converted), expected);
            assert_eq!(shiro_rs_labels_release(&mut converted), 0);
            assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
        }
        assert_eq!(shiro_rs_states_release(&mut states), 0);
    }
}

#[test]
fn arbitrary_time_bits_names_paths_and_late_failures_preserve_outputs() {
    // SAFETY: Readable descriptors/names, unique owners and independent outputs.
    unsafe {
        let mut name = owned("retained\0Unicodeλ".as_bytes());
        let values = [
            ShiroRsLabelInput {
                start: -0.0,
                end: f64::from_bits(1),
                name,
            },
            ShiroRsLabelInput {
                start: f64::from_bits(0x7ff8_0000_0000_0042),
                end: f64::NEG_INFINITY,
                name,
            },
        ];
        let mut owner = null_mut();
        assert_eq!(
            shiro_rs_labels_create(values.as_ptr(), values.len(), &mut owner),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut name), 0);
        for (index, expected) in values.iter().enumerate() {
            let mut info = ShiroRsLabelInfo {
                start: 1.0,
                end: 1.0,
            };
            assert_eq!(shiro_rs_labels_get_info(owner, index, &mut info), 0);
            assert_eq!(info.start.to_bits(), expected.start.to_bits());
            assert_eq!(info.end.to_bits(), expected.end.to_bits());
        }
        let mut snapshot = null_mut();
        assert_eq!(shiro_rs_labels_get_name(owner, 0, &mut snapshot), 0);
        let mut retained = owner;
        let invalid = [
            values[0],
            ShiroRsLabelInput {
                name: null(),
                ..values[1]
            },
        ];
        // The first descriptor must hold a live owner even for a later failure.
        let mut live_name = owned(b"valid");
        let invalid = [
            ShiroRsLabelInput {
                name: live_name,
                ..invalid[0]
            },
            invalid[1],
        ];
        assert_eq!(
            shiro_rs_labels_create(invalid.as_ptr(), 2, &mut retained),
            1
        );
        assert_eq!(retained, owner);
        assert_eq!(shiro_rs_labels_create(null(), usize::MAX, &mut retained), 2);
        assert_eq!(retained, owner);
        let mut bad = owned(b"0 1 aa\n\nx 2 bb");
        assert_eq!(shiro_rs_labels_parse(bad, &mut retained), 3);
        assert_eq!(retained, owner);
        assert_eq!(shiro_rs_bytes_release(&mut bad), 0);
        let mut info = ShiroRsLabelInfo {
            start: 12.0,
            end: 13.0,
        };
        assert_eq!(shiro_rs_labels_get_info(owner, 2, &mut info), 2);
        assert_eq!(
            info,
            ShiroRsLabelInfo {
                start: 12.0,
                end: 13.0
            }
        );
        let mut retained_bytes = snapshot;
        assert_eq!(shiro_rs_labels_get_name(owner, 2, &mut retained_bytes), 2);
        assert_eq!(retained_bytes, snapshot);
        assert_eq!(shiro_rs_labels_release(&mut owner), 0);
        assert_eq!(bytes(snapshot), "retained\0Unicodeλ".as_bytes());
        let mut empty = null_mut();
        assert_eq!(shiro_rs_labels_create(null(), 0, &mut empty), 0);
        let mut count = 99;
        assert_eq!(shiro_rs_labels_length(empty, &mut count), 0);
        assert_eq!(count, 0);
        let mut bad_name = owned(b"bad\tname");
        let labels = [
            ShiroRsLabelInput {
                start: 0.0,
                end: 1.0,
                name: live_name,
            },
            ShiroRsLabelInput {
                start: 1.0,
                end: 2.0,
                name: bad_name,
            },
        ];
        assert_eq!(shiro_rs_labels_create(labels.as_ptr(), 2, &mut owner), 0);
        assert_eq!(shiro_rs_labels_write_bytes(owner, &mut retained_bytes), 3);
        assert_eq!(retained_bytes, snapshot);
        assert_eq!(shiro_rs_labels_release(&mut owner), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bad_name), 0);
        for (filename, expected) in [
            ("dir.name/clip.features.f", "dir.name/clip.features.txt"),
            ("dir.name\\clip", "dir.name\\clip.txt"),
            (".f", ".txt"),
            ("clip", "clip.txt"),
        ] {
            let mut filename = owned(filename.as_bytes());
            let mut suffix = owned(b".txt");
            let mut path = null_mut();
            assert_eq!(shiro_rs_labels_output_path(filename, suffix, &mut path), 0);
            assert_eq!(bytes(path), expected.as_bytes());
            assert_eq!(shiro_rs_bytes_release(&mut path), 0);
            assert_eq!(shiro_rs_bytes_release(&mut suffix), 0);
            assert_eq!(shiro_rs_bytes_release(&mut filename), 0);
        }
        assert_eq!(shiro_rs_labels_release(null_mut()), 1);
        assert_eq!(shiro_rs_labels_clone(empty, null_mut()), 1);
        assert_eq!(shiro_rs_labels_length(empty, null_mut()), 1);
        assert_eq!(shiro_rs_labels_release(&mut empty), 0);
        assert_eq!(shiro_rs_bytes_release(&mut live_name), 0);
        assert_eq!(shiro_rs_bytes_release(&mut snapshot), 0);
    }
}
