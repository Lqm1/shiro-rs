use shiro_rs::labels::{SegmentationDocument, State};
use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

unsafe fn bytes(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Complete readable range and independent output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn copy(data: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live immutable owner and independent output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(data, &mut count), 0);
        let mut output = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(data, 0, output.as_mut_ptr(), count), 0);
        output
    }
}
unsafe fn states(values: &[State]) -> *mut ShiroRsStates {
    // SAFETY: Native values, unique nested owners and initialized descriptors.
    unsafe {
        let mut byte_owners = Vec::new();
        let mut array_owners = Vec::new();
        let mut inputs = Vec::new();
        for value in values {
            let metadata = bytes(&serde_json::to_vec(&value.metadata).unwrap());
            let attributes = bytes(&serde_json::to_vec(&value.attributes).unwrap());
            byte_owners.extend([metadata, attributes]);
            let jumps = match &value.jumps {
                None => null_mut(),
                Some(value) => {
                    let owner = bytes(&serde_json::to_vec(value).unwrap());
                    byte_owners.push(owner);
                    owner
                }
            };
            let mut outputs = null_mut();
            if let Some(value) = &value.outputs {
                assert_eq!(
                    shiro_rs_array_usize_create(value.as_ptr(), value.len(), &mut outputs),
                    0
                );
                array_owners.push(outputs);
            }
            inputs.push(ShiroRsStateInput {
                time: value.time,
                has_duration: u32::from(value.duration.is_some()),
                duration: value.duration.unwrap_or(0),
                outputs,
                jumps,
                metadata,
                attributes,
            });
        }
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_states_create(inputs.as_ptr(), inputs.len(), &mut output),
            0
        );
        for mut owner in byte_owners {
            assert_eq!(shiro_rs_bytes_release(&mut owner), 0);
        }
        for mut owner in array_owners {
            assert_eq!(shiro_rs_array_usize_release(&mut owner), 0);
        }
        output
    }
}

#[test]
fn original_documents_and_full_typed_state_construction_match_every_native_field() {
    for data in [
        include_bytes!("../../../tests/fixtures/utterances-c-initial.json").as_slice(),
        include_bytes!("../../../tests/fixtures/utterances-c-aligned.json").as_slice(),
        include_bytes!("../../../tests/fixtures/align-c-isolated.json").as_slice(),
        include_bytes!("../../../tests/fixtures/init-segmentation.json").as_slice(),
    ] {
        let native: SegmentationDocument = serde_json::from_slice(data).unwrap();
        // SAFETY: Live owners and independent initialized output slots.
        unsafe {
            let mut input = bytes(data);
            let mut parsed = null_mut();
            assert_eq!(shiro_rs_document_read_json(input, &mut parsed), 0);
            assert_eq!(shiro_rs_bytes_release(&mut input), 0);
            let mut length = 99;
            assert_eq!(shiro_rs_document_length(parsed, &mut length), 0);
            assert_eq!(length, native.files.len());
            let mut attributes = null_mut();
            assert_eq!(shiro_rs_document_get_attributes(parsed, &mut attributes), 0);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&copy(attributes)).unwrap(),
                serde_json::to_value(&native.attributes).unwrap()
            );
            let mut files = Vec::new();
            for (index, expected) in native.files.iter().enumerate() {
                let mut original = null_mut();
                assert_eq!(shiro_rs_document_get_file(parsed, index, &mut original), 0);
                let mut snapshot = null_mut();
                assert_eq!(shiro_rs_segmented_file_clone(original, &mut snapshot), 0);
                assert_eq!(shiro_rs_segmented_file_release(&mut original), 0);
                let (mut name, mut attrs, mut actual_states) = (null_mut(), null_mut(), null_mut());
                assert_eq!(shiro_rs_segmented_file_get_filename(snapshot, &mut name), 0);
                assert_eq!(copy(name), expected.filename.as_bytes());
                assert_eq!(
                    shiro_rs_segmented_file_get_attributes(snapshot, &mut attrs),
                    0
                );
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&copy(attrs)).unwrap(),
                    serde_json::to_value(&expected.attributes).unwrap()
                );
                assert_eq!(
                    shiro_rs_segmented_file_get_states(snapshot, &mut actual_states),
                    0
                );
                assert_eq!(shiro_rs_segmented_file_release(&mut snapshot), 0);
                let mut state_json = null_mut();
                assert_eq!(
                    shiro_rs_states_write_json(actual_states, &mut state_json),
                    0
                );
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&copy(state_json)).unwrap(),
                    serde_json::to_value(&expected.states).unwrap()
                );
                let mut typed = states(&expected.states);
                let mut built = null_mut();
                assert_eq!(
                    shiro_rs_segmented_file_create(name, typed, attrs, &mut built),
                    0
                );
                files.push(built);
                assert_eq!(shiro_rs_states_release(&mut typed), 0);
                assert_eq!(shiro_rs_states_release(&mut actual_states), 0);
                for mut value in [name, attrs, state_json] {
                    assert_eq!(shiro_rs_bytes_release(&mut value), 0);
                }
            }
            assert_eq!(shiro_rs_document_release(&mut parsed), 0);
            let inputs: Vec<_> = files.iter().map(|value| value.cast_const()).collect();
            let mut rebuilt = null_mut();
            assert_eq!(
                shiro_rs_document_create(inputs.as_ptr(), inputs.len(), attributes, &mut rebuilt),
                0
            );
            for mut file in files {
                assert_eq!(shiro_rs_segmented_file_release(&mut file), 0);
            }
            assert_eq!(shiro_rs_bytes_release(&mut attributes), 0);
            let mut cloned = null_mut();
            assert_eq!(shiro_rs_document_clone(rebuilt, &mut cloned), 0);
            assert_eq!(shiro_rs_document_release(&mut rebuilt), 0);
            let mut encoded = null_mut();
            assert_eq!(shiro_rs_document_write_json(cloned, &mut encoded), 0);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&copy(encoded)).unwrap(),
                serde_json::to_value(&native).unwrap()
            );
            assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
            assert_eq!(shiro_rs_document_release(&mut cloned), 0);
        }
    }
}

#[test]
fn arbitrary_document_fields_reserved_attributes_and_nonfinite_times_survive_source_release() {
    let state = State {
        time: f64::from_bits(0x7ff8_1234_5678_9abc),
        duration: Some(usize::MAX),
        outputs: Some(vec![0, usize::MAX, usize::MAX - 1]),
        jumps: Some(vec![serde_json::json!([null, -7, {"nested":true}])]),
        metadata: vec![serde_json::json!({"phone":"\u{0}"})],
        attributes: serde_json::from_str(r#"{"time":"shadow","dur":false}"#).unwrap(),
    };
    // SAFETY: Unique owners and independent initialized output storage.
    unsafe {
        let mut typed = states(std::slice::from_ref(&state));
        let mut name = bytes("audio-\u{1f3b5}\0file.wav".as_bytes());
        let mut attrs = bytes(br#"{"filename":"shadow","states":0,"nested":[null,true]}"#);
        let mut top_attrs = bytes(br#"{"file_list":"shadow","nested":{"ok":true}}"#);
        let mut file = null_mut();
        assert_eq!(
            shiro_rs_segmented_file_create(name, typed, attrs, &mut file),
            0
        );
        assert_eq!(shiro_rs_states_release(&mut typed), 0);
        assert_eq!(shiro_rs_bytes_release(&mut name), 0);
        let inputs = [file.cast_const(), file.cast_const()];
        let mut document = null_mut();
        assert_eq!(
            shiro_rs_document_create(inputs.as_ptr(), inputs.len(), top_attrs, &mut document),
            0
        );
        assert_eq!(shiro_rs_segmented_file_release(&mut file), 0);
        let mut cloned = null_mut();
        assert_eq!(shiro_rs_document_clone(document, &mut cloned), 0);
        assert_eq!(shiro_rs_document_release(&mut document), 0);
        let mut count = 99;
        assert_eq!(shiro_rs_document_length(cloned, &mut count), 0);
        assert_eq!(count, 2);
        let mut actual_attrs = null_mut();
        assert_eq!(
            shiro_rs_document_get_attributes(cloned, &mut actual_attrs),
            0
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&copy(actual_attrs)).unwrap(),
            serde_json::from_slice::<serde_json::Value>(&copy(top_attrs)).unwrap()
        );
        assert_eq!(shiro_rs_bytes_release(&mut actual_attrs), 0);
        let mut snapshots = Vec::new();
        for index in 0..2 {
            let mut snapshot = null_mut();
            assert_eq!(shiro_rs_document_get_file(cloned, index, &mut snapshot), 0);
            snapshots.push(snapshot);
        }
        let mut retained_file = snapshots[0];
        assert_eq!(shiro_rs_document_get_file(cloned, 2, &mut retained_file), 2);
        assert_eq!(retained_file, snapshots[0]);
        let mut retained_bytes = attrs;
        assert_eq!(shiro_rs_document_write_json(cloned, &mut retained_bytes), 3);
        assert_eq!(retained_bytes, attrs);
        let mut retained_document = cloned;
        assert_eq!(
            shiro_rs_document_create(null(), 1, top_attrs, &mut retained_document),
            1
        );
        assert_eq!(retained_document, cloned);
        let invalid_files = [null()];
        assert_eq!(
            shiro_rs_document_create(invalid_files.as_ptr(), 1, top_attrs, &mut retained_document),
            1
        );
        assert_eq!(retained_document, cloned);
        let mut malformed = bytes(b"{}");
        assert_eq!(
            shiro_rs_document_read_json(malformed, &mut retained_document),
            3
        );
        assert_eq!(retained_document, cloned);
        assert_eq!(shiro_rs_bytes_release(&mut malformed), 0);
        assert_eq!(shiro_rs_document_release(&mut cloned), 0);
        for mut snapshot in snapshots {
            let (mut name, mut states, mut actual_attrs) = (null_mut(), null_mut(), null_mut());
            assert_eq!(shiro_rs_segmented_file_get_filename(snapshot, &mut name), 0);
            assert_eq!(shiro_rs_segmented_file_get_states(snapshot, &mut states), 0);
            assert_eq!(
                shiro_rs_segmented_file_get_attributes(snapshot, &mut actual_attrs),
                0
            );
            assert_eq!(shiro_rs_segmented_file_release(&mut snapshot), 0);
            assert_eq!(copy(name), "audio-\u{1f3b5}\0file.wav".as_bytes());
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&copy(actual_attrs)).unwrap(),
                serde_json::from_slice::<serde_json::Value>(&copy(attrs)).unwrap()
            );
            let mut info = ShiroRsStateInfo {
                time: 0.0,
                has_duration: 0,
                duration: 0,
                has_outputs: 0,
                has_jumps: 0,
            };
            assert_eq!(shiro_rs_states_get_info(states, 0, &mut info), 0);
            assert_eq!(info.time.to_bits(), state.time.to_bits());
            assert_eq!(
                (
                    info.duration,
                    info.has_duration,
                    info.has_outputs,
                    info.has_jumps
                ),
                (usize::MAX, 1, 1, 1)
            );
            let mut outputs = null_mut();
            assert_eq!(shiro_rs_states_get_outputs(states, 0, &mut outputs), 0);
            let mut values = [0; 3];
            assert_eq!(
                shiro_rs_array_usize_copy(outputs, 0, values.as_mut_ptr(), 3),
                0
            );
            assert_eq!(values, [0, usize::MAX, usize::MAX - 1]);
            assert_eq!(shiro_rs_array_usize_release(&mut outputs), 0);
            let mut invalid_name = bytes(&[0xff]);
            let mut retained = null_mut();
            assert_eq!(
                shiro_rs_segmented_file_create(invalid_name, states, attrs, &mut retained),
                3
            );
            assert!(retained.is_null());
            assert_eq!(shiro_rs_bytes_release(&mut invalid_name), 0);
            assert_eq!(shiro_rs_states_release(&mut states), 0);
            assert_eq!(shiro_rs_bytes_release(&mut name), 0);
            assert_eq!(shiro_rs_bytes_release(&mut actual_attrs), 0);
        }
        assert_eq!(
            shiro_rs_document_create(null(), 0, top_attrs, &mut document),
            0
        );
        assert_eq!(shiro_rs_document_length(document, &mut count), 0);
        assert_eq!(count, 0);
        assert_eq!(shiro_rs_document_release(&mut document), 0);
        assert_eq!(shiro_rs_document_release(&mut document), 0);
        assert_eq!(shiro_rs_bytes_release(&mut attrs), 0);
        assert_eq!(shiro_rs_bytes_release(&mut top_attrs), 0);
    }
}
