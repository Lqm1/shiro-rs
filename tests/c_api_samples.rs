#![cfg(feature = "c-api")]
use shiro_rs::{
    c_api::*,
    dataset,
    definition::ModelDefinition,
    hsmm::{Observation, Segmentation},
    labels::State,
};
use std::ptr::{null, null_mut};

unsafe fn owned(source: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable input and independent initialized output slot.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(source.as_ptr(), source.len(), &mut output) },
        0
    );
    output
}

unsafe fn copied(source: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live readable owner and independent initialized output storage.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(source, &mut count), 0);
        let mut values = vec![0; count];
        assert_eq!(
            shiro_rs_bytes_copy(source, 0, values.as_mut_ptr(), count),
            0
        );
        values
    }
}

#[test]
fn observations_preserve_every_deinterleaved_field_and_source_lifetimes() {
    let bits = [0u32, 0x80000000, 1, 0x80000001, 0x3f800001, 0x3f000000];
    let wire: Vec<_> = bits.into_iter().flat_map(u32::to_le_bytes).collect();
    for raw in [
        wire.as_slice(),
        include_bytes!("fixtures/init-input.bin").as_slice(),
    ] {
        let expected = dataset::read_observation(raw, &[2, 1], 12).unwrap();
        let mut expected_wire = Vec::new();
        expected.write_to(&mut expected_wire).unwrap();
        // SAFETY: Unique live owners and independent initialized output slots.
        unsafe {
            let mut source = owned(raw);
            let mut observation = null_mut();
            let mut clone = null_mut();
            let mut encoded = null_mut();
            assert_eq!(
                shiro_rs_observation_read_rawfloat(
                    source,
                    [2usize, 1].as_ptr(),
                    2,
                    12,
                    &mut observation
                ),
                0
            );
            assert_eq!(shiro_rs_bytes_release(&mut source), 0);
            assert_eq!(shiro_rs_observation_clone(observation, &mut clone), 0);
            assert_eq!(shiro_rs_observation_release(&mut observation), 0);
            assert!(observation.is_null());
            assert_eq!(shiro_rs_observation_release(&mut observation), 0);
            assert_eq!(shiro_rs_observation_write_bytes(clone, &mut encoded), 0);
            assert_eq!(shiro_rs_observation_release(&mut clone), 0);
            let actual = copied(encoded);
            assert_eq!(actual, expected_wire);
            let decoded = Observation::read_from(actual.as_slice()).unwrap();
            assert_eq!(decoded.frames, expected.frames);
            assert_eq!(decoded.streams.len(), expected.streams.len());
            for (actual, expected) in decoded.streams.iter().zip(&expected.streams) {
                assert_eq!(actual.dimensions, expected.dimensions);
                assert_eq!(
                    actual
                        .values
                        .iter()
                        .map(|x| x.to_bits())
                        .collect::<Vec<_>>(),
                    expected
                        .values
                        .iter()
                        .map(|x| x.to_bits())
                        .collect::<Vec<_>>()
                );
            }
            assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
            let mut definition = owned(include_bytes!("fixtures/init-definition.json"));
            let mut model = null_mut();
            assert_eq!(shiro_rs_model_from_definition(definition, &mut model), 0);
            assert_eq!(shiro_rs_bytes_release(&mut definition), 0);
            source = owned(raw);
            assert_eq!(
                shiro_rs_observation_from_model_rawfloat(source, model, 12, &mut observation),
                0
            );
            assert_eq!(shiro_rs_model_release(&mut model), 0);
            assert_eq!(shiro_rs_bytes_release(&mut source), 0);
            assert_eq!(
                shiro_rs_observation_write_bytes(observation, &mut encoded),
                0
            );
            assert_eq!(copied(encoded), expected_wire);
            assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
            assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        }
    }
}

#[test]
fn states_preserve_optional_fields_nested_metadata_and_complete_segmentation() {
    let json = br#"[{"time":1.75,"dur":0,"out":[0,0],"jmp":[{"d":0,"p":0.123456789,"edge":"keep"},{"d":1,"p":0.9}],"ext":["aa",0,{"nested":[true,null,5]}],"custom":{"value":"keep"}},{"time":4.125,"dur":1,"out":[1,1],"jmp":[],"ext":["aa",1,"tail"]}]"#;
    let expected: Vec<State> = serde_json::from_slice(json).unwrap();
    let native_model =
        serde_json::from_slice::<ModelDefinition>(include_bytes!("fixtures/init-definition.json"))
            .unwrap()
            .build()
            .unwrap();
    let expected_segmentation = dataset::read_segmentation(&expected, &native_model).unwrap();
    let mut expected_wire = Vec::new();
    expected_segmentation.write_to(&mut expected_wire).unwrap();
    // SAFETY: Unique owners and independent output slots; outputs used only while live.
    unsafe {
        let mut bytes = owned(json);
        let mut states = null_mut();
        let mut clone = null_mut();
        let mut output = null_mut();
        assert_eq!(shiro_rs_states_read_json(bytes, &mut states), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        assert_eq!(shiro_rs_states_clone(states, &mut clone), 0);
        assert_eq!(shiro_rs_states_release(&mut states), 0);
        assert!(states.is_null());
        assert_eq!(shiro_rs_states_release(&mut states), 0);
        assert_eq!(shiro_rs_states_write_json(clone, &mut output), 0);
        assert_eq!(copied(output), serde_json::to_vec(&expected).unwrap());
        assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        let mut definition = owned(include_bytes!("fixtures/init-definition.json"));
        let mut model = null_mut();
        assert_eq!(shiro_rs_model_from_definition(definition, &mut model), 0);
        assert_eq!(shiro_rs_bytes_release(&mut definition), 0);
        assert_eq!(
            shiro_rs_states_segmentation_bytes(clone, model, &mut output),
            0
        );
        assert_eq!(copied(output), expected_wire);
        assert_eq!(
            Segmentation::read_from(copied(output).as_slice()).unwrap(),
            expected_segmentation
        );
        for invalid in [
            br#"[{"time":1,"out":[0,0]}]"#.as_slice(),
            br#"[{"time":1,"dur":0,"out":[0]}]"#,
            br#"[{"time":1,"dur":99,"out":[0,0]}]"#,
            br#"[{"time":-1,"dur":0,"out":[0,0]}]"#,
            br#"[{"time":1,"dur":0,"out":[0,0],"jmp":[{"d":0,"p":1.1}]}]"#,
        ] {
            let mut invalid_json = owned(invalid);
            let mut invalid_states = null_mut();
            assert_eq!(
                shiro_rs_states_read_json(invalid_json, &mut invalid_states),
                0
            );
            let mut retained_output = output;
            assert_eq!(
                shiro_rs_states_segmentation_bytes(invalid_states, model, &mut retained_output),
                3
            );
            assert_eq!(retained_output, output);
            assert_eq!(copied(output), expected_wire);
            assert_eq!(shiro_rs_states_release(&mut invalid_states), 0);
            assert_eq!(shiro_rs_bytes_release(&mut invalid_json), 0);
        }
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert_eq!(shiro_rs_states_release(&mut clone), 0);
        assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        for json in [
            b"[]".as_slice(),
            b"[{\"custom\":null}]",
            b"[{\"dur\":null,\"out\":null,\"jmp\":null,\"ext\":[null]}]",
        ] {
            bytes = owned(json);
            let native: Vec<State> = serde_json::from_slice(json).unwrap();
            assert_eq!(shiro_rs_states_read_json(bytes, &mut states), 0);
            assert_eq!(shiro_rs_states_write_json(states, &mut output), 0);
            assert_eq!(copied(output), serde_json::to_vec(&native).unwrap());
            assert_eq!(shiro_rs_bytes_release(&mut output), 0);
            assert_eq!(shiro_rs_states_release(&mut states), 0);
            assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        }
    }
}

#[test]
fn invalid_samples_retain_every_output_and_enforce_native_frame_limits() {
    // SAFETY: Unique live owners and independent initialized sentinel slots.
    unsafe {
        let raw = include_bytes!("fixtures/init-input.bin");
        let dims = [2usize, 1];
        let mut bytes = owned(raw);
        let mut observation = null_mut();
        assert_eq!(
            shiro_rs_observation_read_rawfloat(bytes, dims.as_ptr(), 2, 12, &mut observation),
            0
        );
        for frames in 0..12 {
            let mut retained = observation;
            assert_eq!(
                shiro_rs_observation_read_rawfloat(bytes, dims.as_ptr(), 2, frames, &mut retained),
                3
            );
            assert_eq!(retained, observation);
        }
        for invalid_dims in [&[][..], &[0usize][..], &[i32::MAX as usize + 1][..]] {
            let mut retained = observation;
            assert_eq!(
                shiro_rs_observation_read_rawfloat(
                    bytes,
                    invalid_dims.as_ptr(),
                    invalid_dims.len(),
                    12,
                    &mut retained
                ),
                3
            );
            assert_eq!(retained, observation);
        }
        let mut retained_observation = observation;
        assert_eq!(
            shiro_rs_observation_read_rawfloat(bytes, null(), 1, 12, &mut retained_observation),
            1
        );
        assert_eq!(
            shiro_rs_observation_read_rawfloat(
                bytes,
                null(),
                usize::MAX,
                12,
                &mut retained_observation
            ),
            2
        );
        assert_eq!(
            shiro_rs_observation_clone(null(), &mut retained_observation),
            1
        );
        assert_eq!(shiro_rs_observation_clone(observation, null_mut()), 1);
        assert_eq!(shiro_rs_observation_write_bytes(observation, null_mut()), 1);
        assert_eq!(
            shiro_rs_observation_from_model_rawfloat(bytes, null(), 12, &mut retained_observation),
            1
        );
        assert_eq!(retained_observation, observation);
        assert_eq!(shiro_rs_observation_release(null_mut()), 1);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        for end in [1, raw.len() - 1, raw.len() - 4] {
            bytes = owned(&raw[..end]);
            let mut retained = observation;
            assert_eq!(
                shiro_rs_observation_read_rawfloat(bytes, dims.as_ptr(), 2, 12, &mut retained),
                3
            );
            assert_eq!(retained, observation);
            assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        }
        assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        bytes = owned(b"[]");
        let mut states = null_mut();
        assert_eq!(shiro_rs_states_read_json(bytes, &mut states), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        for invalid in [
            b"{}".as_slice(),
            b"[",
            b"[] []",
            b"[{\"out\":[-1]}]",
            b"[{\"time\":null}]",
        ] {
            bytes = owned(invalid);
            let mut retained = states;
            assert_eq!(shiro_rs_states_read_json(bytes, &mut retained), 3);
            assert_eq!(retained, states);
            assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        }
        let mut retained_states = states;
        assert_eq!(shiro_rs_states_read_json(null(), &mut retained_states), 1);
        assert_eq!(shiro_rs_states_clone(null(), &mut retained_states), 1);
        assert_eq!(shiro_rs_states_clone(states, null_mut()), 1);
        assert_eq!(shiro_rs_states_write_json(states, null_mut()), 1);
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_states_segmentation_bytes(states, null(), &mut output),
            1
        );
        assert!(output.is_null());
        assert_eq!(retained_states, states);
        assert_eq!(shiro_rs_states_release(null_mut()), 1);
        assert_eq!(shiro_rs_states_release(&mut states), 0);
    }
}
