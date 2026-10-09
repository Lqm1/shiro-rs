use shiro_rs::labels::SegmentationDocument;
use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

unsafe fn bytes(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable input and independent owner slot.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn json(value: *const ShiroRsBytes) -> serde_json::Value {
    // SAFETY: Live readable owner and independent initialized output.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(value, &mut count), 0);
        let mut output = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(value, 0, output.as_mut_ptr(), count), 0);
        serde_json::from_slice(&output).unwrap()
    }
}

#[test]
fn original_states_retain_every_scalar_optional_field_and_metadata() {
    for data in [
        include_bytes!("../../../tests/fixtures/utterances-c-initial.json").as_slice(),
        include_bytes!("../../../tests/fixtures/utterances-c-aligned.json").as_slice(),
        include_bytes!("../../../tests/fixtures/align-c-isolated.json").as_slice(),
    ] {
        let document: SegmentationDocument = serde_json::from_slice(data).unwrap();
        for file in document.files {
            // SAFETY: Unique owners, immutable input, independent output storage.
            unsafe {
                let mut source = bytes(&serde_json::to_vec(&file.states).unwrap());
                let mut states = null_mut();
                assert_eq!(shiro_rs_states_read_json(source, &mut states), 0);
                assert_eq!(shiro_rs_bytes_release(&mut source), 0);
                let mut count = 0;
                assert_eq!(shiro_rs_states_length(states, &mut count), 0);
                assert_eq!(count, file.states.len());
                for (index, expected) in file.states.iter().enumerate() {
                    let mut info = ShiroRsStateInfo {
                        time: 99.0,
                        has_duration: 0,
                        duration: 0,
                        has_outputs: 0,
                        has_jumps: 0,
                    };
                    assert_eq!(shiro_rs_states_get_info(states, index, &mut info), 0);
                    assert_eq!(
                        (
                            info.time.to_bits(),
                            info.has_duration,
                            info.duration,
                            info.has_outputs,
                            info.has_jumps
                        ),
                        (
                            expected.time.to_bits(),
                            u32::from(expected.duration.is_some()),
                            expected.duration.unwrap_or(0),
                            u32::from(expected.outputs.is_some()),
                            u32::from(expected.jumps.is_some())
                        )
                    );
                    let mut outputs = null_mut();
                    let status = shiro_rs_states_get_outputs(states, index, &mut outputs);
                    if let Some(expected) = &expected.outputs {
                        assert_eq!(status, 0);
                        let mut count = 0;
                        assert_eq!(shiro_rs_array_usize_length(outputs, &mut count), 0);
                        let mut actual = vec![0; count];
                        assert_eq!(
                            shiro_rs_array_usize_copy(outputs, 0, actual.as_mut_ptr(), count),
                            0
                        );
                        assert_eq!(&actual, expected);
                        assert_eq!(shiro_rs_array_usize_release(&mut outputs), 0);
                    } else {
                        assert_eq!(status, 2);
                        assert!(outputs.is_null());
                    }
                    for field in 0..3 {
                        let mut output = null_mut();
                        let status =
                            shiro_rs_states_get_json_field(states, index, field, &mut output);
                        if field == 0 && expected.jumps.is_none() {
                            assert_eq!(status, 2);
                            continue;
                        }
                        assert_eq!(status, 0);
                        let expected = match field {
                            0 => serde_json::to_value(&expected.jumps).unwrap(),
                            1 => serde_json::to_value(&expected.metadata).unwrap(),
                            _ => serde_json::to_value(&expected.attributes).unwrap(),
                        };
                        assert_eq!(json(output), expected);
                        assert_eq!(shiro_rs_bytes_release(&mut output), 0);
                    }
                }
                assert_eq!(shiro_rs_states_release(&mut states), 0);
            }
        }
    }
}

#[test]
fn typed_states_preserve_arbitrary_time_bits_optionals_reserved_attributes_and_lifetimes() {
    let patterns = [
        0x8000_0000_0000_0000,
        0x7ff8_1234_5678_9abc,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        1,
    ];
    // SAFETY: Unique owners, live immutable input ranges, independent outputs.
    unsafe {
        let mut metadata = bytes(br#"["phone",7,{"nested":[null,true,"\u0000"]}]"#);
        let mut attributes = bytes(
            br#"{"time":"attribute","dur":[1],"out":false,"ext":{"nested":true},"jmp":null}"#,
        );
        let mut jumps = bytes(b"[]");
        let mut outputs = null_mut();
        assert_eq!(shiro_rs_array_usize_create(null(), 0, &mut outputs), 0);
        let inputs: Vec<_> = patterns
            .iter()
            .enumerate()
            .map(|(index, &bits)| ShiroRsStateInput {
                time: f64::from_bits(bits),
                has_duration: u32::from(index != 0),
                duration: if index == 1 { 0 } else { usize::MAX },
                outputs: if index == 0 { null() } else { outputs },
                jumps: if index == 0 { null() } else { jumps },
                metadata,
                attributes,
            })
            .collect();
        let mut states = null_mut();
        assert_eq!(
            shiro_rs_states_create(inputs.as_ptr(), inputs.len(), &mut states),
            0
        );
        let expected_metadata = json(metadata);
        let expected_attributes = json(attributes);
        assert_eq!(shiro_rs_bytes_release(&mut metadata), 0);
        assert_eq!(shiro_rs_bytes_release(&mut attributes), 0);
        assert_eq!(shiro_rs_bytes_release(&mut jumps), 0);
        assert_eq!(shiro_rs_array_usize_release(&mut outputs), 0);
        let mut cloned = null_mut();
        assert_eq!(shiro_rs_states_clone(states, &mut cloned), 0);
        assert_eq!(shiro_rs_states_release(&mut states), 0);
        for (index, &pattern) in patterns.iter().enumerate() {
            let mut info = ShiroRsStateInfo {
                time: 0.0,
                has_duration: 0,
                duration: 0,
                has_outputs: 0,
                has_jumps: 0,
            };
            assert_eq!(shiro_rs_states_get_info(cloned, index, &mut info), 0);
            assert_eq!(info.time.to_bits(), pattern);
            assert_eq!(
                (
                    info.has_duration,
                    info.duration,
                    info.has_outputs,
                    info.has_jumps
                ),
                (
                    u32::from(index != 0),
                    if index <= 1 { 0 } else { usize::MAX },
                    u32::from(index != 0),
                    u32::from(index != 0)
                )
            );
            let mut outputs = null_mut();
            assert_eq!(
                shiro_rs_states_get_outputs(cloned, index, &mut outputs),
                if index == 0 { 2 } else { 0 }
            );
            if index != 0 {
                let mut count = 99;
                assert_eq!(shiro_rs_array_usize_length(outputs, &mut count), 0);
                assert_eq!(count, 0);
                assert_eq!(shiro_rs_array_usize_release(&mut outputs), 0);
            }
            for field in 0..3 {
                let mut output = null_mut();
                assert_eq!(
                    shiro_rs_states_get_json_field(cloned, index, field, &mut output),
                    if index == 0 && field == 0 { 2 } else { 0 }
                );
                if index == 0 && field == 0 {
                    continue;
                }
                assert_eq!(
                    json(output),
                    match field {
                        0 => serde_json::json!([]),
                        1 => expected_metadata.clone(),
                        _ => expected_attributes.clone(),
                    }
                );
                assert_eq!(shiro_rs_bytes_release(&mut output), 0);
            }
        }
        let mut sentinel = bytes(b"{}");
        let mut retained = sentinel;
        assert_eq!(shiro_rs_states_write_json(cloned, &mut retained), 3);
        assert_eq!(retained, sentinel);
        assert_eq!(
            shiro_rs_states_get_json_field(cloned, 0, 3, &mut retained),
            2
        );
        assert_eq!(retained, sentinel);
        assert_eq!(
            shiro_rs_states_get_json_field(cloned, usize::MAX, 1, &mut retained),
            2
        );
        assert_eq!(retained, sentinel);
        let invalid = ShiroRsStateInput {
            time: 0.0,
            has_duration: 2,
            duration: 0,
            outputs: null(),
            jumps: null(),
            metadata: sentinel,
            attributes: sentinel,
        };
        let mut retained_states = cloned;
        assert_eq!(shiro_rs_states_create(&invalid, 1, &mut retained_states), 2);
        assert_eq!(retained_states, cloned);
        assert_eq!(shiro_rs_states_create(null(), 1, &mut retained_states), 1);
        assert_eq!(retained_states, cloned);
        let malformed = ShiroRsStateInput {
            time: 0.0,
            has_duration: 0,
            duration: 0,
            outputs: null(),
            jumps: null(),
            metadata: sentinel,
            attributes: sentinel,
        };
        assert_eq!(
            shiro_rs_states_create(&malformed, 1, &mut retained_states),
            3
        );
        assert_eq!(retained_states, cloned);
        assert_eq!(shiro_rs_bytes_release(&mut sentinel), 0);
        assert_eq!(shiro_rs_states_release(&mut cloned), 0);
        assert_eq!(shiro_rs_states_create(null(), 0, &mut states), 0);
        let mut count = 99;
        assert_eq!(shiro_rs_states_length(states, &mut count), 0);
        assert_eq!(count, 0);
        assert_eq!(shiro_rs_states_release(&mut states), 0);
    }
}
