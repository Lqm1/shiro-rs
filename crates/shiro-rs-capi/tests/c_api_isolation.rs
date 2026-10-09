use serde_json::json;
use shiro_rs::{
    dataset,
    hsmm::Model,
    labels::{SegmentationDocument, State},
};
use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

unsafe fn owned(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable input and independent initialized output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn bytes(owner: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live immutable owner and independent output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(owner, &mut count), 0);
        let mut output = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(owner, 0, output.as_mut_ptr(), count), 0);
        output
    }
}
unsafe fn inputs(
    states: &[State],
) -> (
    *mut ShiroRsModel,
    *mut ShiroRsObservation,
    *mut ShiroRsStates,
) {
    // SAFETY: Unique input owners and independent output slots.
    unsafe {
        let mut encoded = owned(include_bytes!(
            "../../../tests/fixtures/init-c-aligned.hsmm"
        ));
        let mut model = null_mut();
        assert_eq!(
            shiro_rs_model_read_bytes(encoded, 16 * 1024 * 1024, &mut model),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        encoded = owned(include_bytes!("../../../tests/fixtures/init-input.bin"));
        let mut observation = null_mut();
        assert_eq!(
            shiro_rs_observation_from_model_rawfloat(encoded, model, 12, &mut observation),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        encoded = owned(&serde_json::to_vec(states).unwrap());
        let mut owner = null_mut();
        assert_eq!(shiro_rs_states_read_json(encoded, &mut owner), 0);
        assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        (model, observation, owner)
    }
}
fn states() -> Vec<State> {
    let document: SegmentationDocument = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/align-c-isolated.json"
    ))
    .unwrap();
    document.files.into_iter().next().unwrap().states
}

#[test]
fn complete_original_positions_samples_metadata_and_jump_filtering_match_native() {
    let native_model =
        Model::read_from(include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm").as_slice())
            .unwrap();
    let observation = dataset::read_observation(
        include_bytes!("../../../tests/fixtures/init-input.bin").as_slice(),
        &dataset::dimensions(&native_model).unwrap(),
        12,
    )
    .unwrap();
    for enhanced in [false, true] {
        let mut states = states();
        if enhanced {
            states[0].jumps = Some(vec![json!({"d":1,"p":0.9}), json!({"d":4,"p":0.1})]);
            states[0]
                .attributes
                .insert("extra".into(), json!({"nested":[1,null,"retained"]}));
            states[3].metadata.push(json!({"retained":true}));
            states[5].time = 40.0;
        }
        let expected = dataset::isolated_groups(&native_model, &observation, &states).unwrap();
        assert_eq!(expected.len(), 2);
        assert_eq!((expected[1].first_state, expected[1].first_frame), (3, 6));
        // SAFETY: Unique owners, immutable inputs and independent outputs.
        unsafe {
            let (mut model, mut observation, mut source_states) = inputs(&states);
            let mut groups = null_mut();
            assert_eq!(
                shiro_rs_isolated_groups(model, observation, source_states, &mut groups),
                0
            );
            let mut clone = null_mut();
            assert_eq!(shiro_rs_isolated_groups_clone(groups, &mut clone), 0);
            assert_eq!(shiro_rs_isolated_groups_release(&mut groups), 0);
            let mut count = 0;
            assert_eq!(shiro_rs_isolated_groups_length(clone, &mut count), 0);
            assert_eq!(count, expected.len());
            let mut snapshots = Vec::new();
            for (index, expected) in expected.iter().enumerate() {
                let mut info = ShiroRsIsolatedGroupInfo {
                    first_state: 99,
                    first_frame: 99,
                };
                assert_eq!(
                    shiro_rs_isolated_groups_get_info(clone, index, &mut info),
                    0
                );
                assert_eq!(
                    (info.first_state, info.first_frame),
                    (expected.first_state, expected.first_frame)
                );
                let mut sample = null_mut();
                let mut local = null_mut();
                assert_eq!(
                    shiro_rs_isolated_groups_get_observation(clone, index, &mut sample),
                    0
                );
                assert_eq!(
                    shiro_rs_isolated_groups_get_states(clone, index, &mut local),
                    0
                );
                snapshots.push((sample, local));
            }
            assert_eq!(shiro_rs_isolated_groups_release(&mut clone), 0);
            for ((mut sample, mut local), expected) in snapshots.into_iter().zip(expected) {
                let mut wire = null_mut();
                assert_eq!(shiro_rs_observation_write_bytes(sample, &mut wire), 0);
                let mut expected_wire = Vec::new();
                expected.observation.write_to(&mut expected_wire).unwrap();
                assert_eq!(bytes(wire), expected_wire);
                assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
                assert_eq!(shiro_rs_states_write_json(local, &mut wire), 0);
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&bytes(wire)).unwrap(),
                    serde_json::to_value(expected.states).unwrap()
                );
                assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
                assert_eq!(shiro_rs_observation_release(&mut sample), 0);
                assert_eq!(shiro_rs_states_release(&mut local), 0);
            }
            let mut wire = null_mut();
            assert_eq!(shiro_rs_states_write_json(source_states, &mut wire), 0);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&bytes(wire)).unwrap(),
                serde_json::to_value(states).unwrap()
            );
            assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
            assert_eq!(shiro_rs_model_release(&mut model), 0);
            assert_eq!(shiro_rs_observation_release(&mut observation), 0);
            assert_eq!(shiro_rs_states_release(&mut source_states), 0);
        }
    }
}

#[test]
fn arbitrary_public_group_fields_repeated_inputs_and_failures_retain_owners() {
    // SAFETY: Readable descriptors/live owners and independent output slots.
    unsafe {
        let (mut model, mut observation, mut states_owner) = inputs(&states());
        let descriptor = ShiroRsIsolatedGroupInput {
            first_state: usize::MAX,
            first_frame: usize::MAX - 1,
            observation,
            states: states_owner,
        };
        let mut groups = null_mut();
        assert_eq!(
            shiro_rs_isolated_groups_create([descriptor, descriptor].as_ptr(), 2, &mut groups),
            0
        );
        let mut info = ShiroRsIsolatedGroupInfo {
            first_state: 0,
            first_frame: 0,
        };
        assert_eq!(shiro_rs_isolated_groups_get_info(groups, 1, &mut info), 0);
        assert_eq!(
            info,
            ShiroRsIsolatedGroupInfo {
                first_state: usize::MAX,
                first_frame: usize::MAX - 1
            }
        );
        assert_eq!(shiro_rs_isolated_groups_get_info(groups, 2, &mut info), 2);
        assert_eq!(info.first_state, usize::MAX);
        let mut retained = groups;
        let invalid = [
            descriptor,
            ShiroRsIsolatedGroupInput {
                states: null(),
                ..descriptor
            },
        ];
        assert_eq!(
            shiro_rs_isolated_groups_create(invalid.as_ptr(), 2, &mut retained),
            1
        );
        assert_eq!(retained, groups);
        assert_eq!(
            shiro_rs_isolated_groups_create(null(), usize::MAX, &mut retained),
            2
        );
        assert_eq!(retained, groups);
        let mut invalid_states = states();
        for state in &mut invalid_states[3..] {
            state.time = 6.0;
        }
        let mut encoded = owned(&serde_json::to_vec(&invalid_states).unwrap());
        let mut bad = null_mut();
        assert_eq!(shiro_rs_states_read_json(encoded, &mut bad), 0);
        assert_eq!(
            shiro_rs_isolated_groups(model, observation, bad, &mut retained),
            3
        );
        assert_eq!(retained, groups);
        assert_eq!(shiro_rs_states_release(&mut bad), 0);
        assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        encoded = owned(b"[]");
        assert_eq!(shiro_rs_states_read_json(encoded, &mut bad), 0);
        assert_eq!(
            shiro_rs_isolated_groups(model, observation, bad, &mut retained),
            3
        );
        assert_eq!(retained, groups);
        assert_eq!(shiro_rs_states_release(&mut bad), 0);
        assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        let mut sample = null_mut();
        let mut local = null_mut();
        assert_eq!(
            shiro_rs_isolated_groups_get_observation(groups, 0, &mut sample),
            0
        );
        assert_eq!(
            shiro_rs_isolated_groups_get_states(groups, 0, &mut local),
            0
        );
        let mut expected_wire = null_mut();
        assert_eq!(
            shiro_rs_observation_write_bytes(observation, &mut expected_wire),
            0
        );
        let expected_observation = bytes(expected_wire);
        assert_eq!(shiro_rs_bytes_release(&mut expected_wire), 0);
        assert_eq!(shiro_rs_isolated_groups_release(&mut groups), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        assert_eq!(shiro_rs_states_release(&mut states_owner), 0);
        let mut wire = null_mut();
        assert_eq!(shiro_rs_observation_write_bytes(sample, &mut wire), 0);
        assert_eq!(bytes(wire), expected_observation);
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
        assert_eq!(shiro_rs_states_write_json(local, &mut wire), 0);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes(wire)).unwrap(),
            serde_json::to_value(states()).unwrap()
        );
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
        assert_eq!(shiro_rs_observation_release(&mut sample), 0);
        assert_eq!(shiro_rs_states_release(&mut local), 0);
        assert_eq!(shiro_rs_isolated_groups_create(null(), 0, &mut groups), 0);
        let mut count = 99;
        assert_eq!(shiro_rs_isolated_groups_length(groups, &mut count), 0);
        assert_eq!(count, 0);
        assert_eq!(shiro_rs_isolated_groups_clone(groups, null_mut()), 1);
        assert_eq!(shiro_rs_isolated_groups_release(null_mut()), 1);
        assert_eq!(shiro_rs_isolated_groups_release(&mut groups), 0);
    }
}
