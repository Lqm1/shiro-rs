#![cfg(feature = "c-api")]
use shiro_rs::{c_api::*, definition::ModelDefinition};
use std::ptr::{null, null_mut};

unsafe fn bytes(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable input and independent output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn copy(data: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live input and independent initialized buffers.
    unsafe {
        let mut length = 0;
        assert_eq!(shiro_rs_bytes_length(data, &mut length), 0);
        let mut output = vec![0; length];
        assert_eq!(shiro_rs_bytes_copy(data, 0, output.as_mut_ptr(), length), 0);
        output
    }
}

#[test]
fn original_definitions_defaults_and_built_models_match_native() {
    for data in [
        include_bytes!("fixtures/modeldef.json").as_slice(),
        include_bytes!("fixtures/init-definition.json").as_slice(),
        include_bytes!("fixtures/utterances-c-definition.json").as_slice(),
    ] {
        let native: ModelDefinition = serde_json::from_slice(data).unwrap();
        // SAFETY: Unique owners and independent initialized storage throughout.
        unsafe {
            let mut input = bytes(data);
            let mut definition = null_mut();
            assert_eq!(shiro_rs_definition_read_json(input, &mut definition), 0);
            assert_eq!(shiro_rs_bytes_release(&mut input), 0);
            let mut info = ShiroRsDefinitionInfo {
                duration_states: 0,
                streams: 0,
                duration_constraints: 0,
            };
            assert_eq!(shiro_rs_definition_get_info(definition, &mut info), 0);
            assert_eq!(
                (
                    info.duration_states,
                    info.streams,
                    info.duration_constraints
                ),
                (
                    native.duration_states,
                    native.streams.len(),
                    native.duration_constraints.len()
                )
            );
            for (index, value) in native.streams.iter().enumerate() {
                let mut output = ShiroRsStreamDefinition {
                    states: 0,
                    dimensions: 0,
                    mixtures: 0,
                    weight: 0.0,
                };
                assert_eq!(
                    shiro_rs_definition_get_stream(definition, index, &mut output),
                    0
                );
                assert_eq!(
                    (
                        output.states,
                        output.dimensions,
                        output.mixtures,
                        output.weight.to_bits()
                    ),
                    (
                        value.states,
                        value.dimensions,
                        value.mixtures,
                        value.weight.to_bits()
                    )
                );
            }
            for (index, value) in native.duration_constraints.iter().enumerate() {
                let mut output = ShiroRsDurationConstraint {
                    index: 0,
                    has_minimum: 0,
                    minimum: 0,
                    has_maximum: 0,
                    maximum: 0,
                };
                assert_eq!(
                    shiro_rs_definition_get_constraint(definition, index, &mut output),
                    0
                );
                assert_eq!(
                    (
                        output.index,
                        output.has_minimum,
                        output.minimum,
                        output.has_maximum,
                        output.maximum
                    ),
                    (
                        value.index,
                        u32::from(value.minimum.is_some()),
                        value.minimum.unwrap_or(0),
                        u32::from(value.maximum.is_some()),
                        value.maximum.unwrap_or(0)
                    )
                );
            }
            let mut json = null_mut();
            assert_eq!(shiro_rs_definition_write_json(definition, &mut json), 0);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&copy(json)).unwrap(),
                serde_json::to_value(&native).unwrap()
            );
            let mut model = null_mut();
            assert_eq!(shiro_rs_definition_build(definition, &mut model), 0);
            let mut wire = null_mut();
            assert_eq!(shiro_rs_model_write_bytes(model, 0, &mut wire), 0);
            let mut expected = Vec::new();
            native.build().unwrap().write_to(&mut expected).unwrap();
            assert_eq!(copy(wire), expected);
            if data == include_bytes!("fixtures/modeldef.json") {
                assert_eq!(copy(wire), include_bytes!("fixtures/empty-c.hsmm"));
            }
            if data == include_bytes!("fixtures/utterances-c-definition.json") {
                assert_eq!(
                    copy(wire),
                    include_bytes!("fixtures/utterances-c-uninit.hsmm")
                );
            }
            assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
            assert_eq!(shiro_rs_bytes_release(&mut json), 0);
            assert_eq!(shiro_rs_model_release(&mut model), 0);
            assert_eq!(shiro_rs_definition_release(&mut definition), 0);
        }
    }
}

#[test]
fn arbitrary_definition_fields_bits_and_optional_constraints_survive_parent_release() {
    let patterns = [0x8000_0000, 0x7fc1_2345, 0x7f80_0000, 0xff80_0000, 1];
    let streams: Vec<_> = patterns
        .into_iter()
        .map(|bits| ShiroRsStreamDefinition {
            states: usize::MAX,
            dimensions: 0,
            mixtures: usize::MAX - 1,
            weight: f32::from_bits(bits),
        })
        .collect();
    let constraints = [
        ShiroRsDurationConstraint {
            index: usize::MAX,
            has_minimum: 0,
            minimum: 99,
            has_maximum: 1,
            maximum: 0,
        },
        ShiroRsDurationConstraint {
            index: 0,
            has_minimum: 1,
            minimum: i32::MIN,
            has_maximum: 0,
            maximum: 99,
        },
        ShiroRsDurationConstraint {
            index: 1,
            has_minimum: 1,
            minimum: 0,
            has_maximum: 1,
            maximum: i32::MAX,
        },
    ];
    // SAFETY: Complete immutable input arrays and unique owner slots.
    unsafe {
        let mut definition = null_mut();
        assert_eq!(
            shiro_rs_definition_create(
                usize::MAX,
                streams.as_ptr(),
                streams.len(),
                constraints.as_ptr(),
                constraints.len(),
                &mut definition
            ),
            0
        );
        let mut cloned = null_mut();
        assert_eq!(shiro_rs_definition_clone(definition, &mut cloned), 0);
        assert_eq!(shiro_rs_definition_release(&mut definition), 0);
        let mut info = ShiroRsDefinitionInfo {
            duration_states: 0,
            streams: 0,
            duration_constraints: 0,
        };
        assert_eq!(shiro_rs_definition_get_info(cloned, &mut info), 0);
        assert_eq!(info.duration_states, usize::MAX);
        assert_eq!(info.streams, streams.len());
        assert_eq!(info.duration_constraints, constraints.len());
        for (index, expected) in streams.iter().enumerate() {
            let mut actual = streams[0];
            assert_eq!(
                shiro_rs_definition_get_stream(cloned, index, &mut actual),
                0
            );
            assert_eq!(
                (
                    actual.states,
                    actual.dimensions,
                    actual.mixtures,
                    actual.weight.to_bits()
                ),
                (
                    expected.states,
                    expected.dimensions,
                    expected.mixtures,
                    expected.weight.to_bits()
                )
            );
        }
        for (index, expected) in constraints.iter().enumerate() {
            let mut actual = constraints[0];
            assert_eq!(
                shiro_rs_definition_get_constraint(cloned, index, &mut actual),
                0
            );
            assert_eq!(
                actual,
                ShiroRsDurationConstraint {
                    minimum: if expected.has_minimum == 0 {
                        0
                    } else {
                        expected.minimum
                    },
                    maximum: if expected.has_maximum == 0 {
                        0
                    } else {
                        expected.maximum
                    },
                    ..*expected
                }
            );
        }
        let mut sentinel = bytes(b"retained");
        let mut retained = sentinel;
        assert_eq!(shiro_rs_definition_write_json(cloned, &mut retained), 3);
        assert_eq!(retained, sentinel);
        assert_eq!(copy(sentinel), b"retained");
        assert_eq!(shiro_rs_bytes_release(&mut sentinel), 0);
        let mut output = null_mut();
        assert_eq!(shiro_rs_definition_build(cloned, &mut output), 3);
        assert!(output.is_null());
        let mut stream = streams[0];
        assert_eq!(
            shiro_rs_definition_get_stream(cloned, usize::MAX, &mut stream),
            2
        );
        assert_eq!(stream.weight.to_bits(), streams[0].weight.to_bits());
        let invalid = ShiroRsDurationConstraint {
            has_maximum: 2,
            ..constraints[0]
        };
        let mut retained = cloned;
        assert_eq!(
            shiro_rs_definition_create(0, null(), 0, &invalid, 1, &mut retained),
            2
        );
        assert_eq!(retained, cloned);
        assert_eq!(
            shiro_rs_definition_create(0, null(), 1, null(), 0, &mut retained),
            1
        );
        assert_eq!(retained, cloned);
        let mut malformed = bytes(b"{}");
        assert_eq!(shiro_rs_definition_read_json(malformed, &mut retained), 3);
        assert_eq!(retained, cloned);
        assert_eq!(shiro_rs_bytes_release(&mut malformed), 0);
        assert_eq!(shiro_rs_definition_release(&mut cloned), 0);
        assert_eq!(shiro_rs_definition_release(&mut cloned), 0);
        assert!(cloned.is_null());
        let mut empty = null_mut();
        assert_eq!(
            shiro_rs_definition_create(0, null(), 0, null(), 0, &mut empty),
            0
        );
        assert_eq!(shiro_rs_definition_get_info(empty, &mut info), 0);
        assert_eq!(
            (
                info.duration_states,
                info.streams,
                info.duration_constraints
            ),
            (0, 0, 0)
        );
        assert_eq!(shiro_rs_definition_release(&mut empty), 0);
    }
}
