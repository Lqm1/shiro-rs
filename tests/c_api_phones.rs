#![cfg(feature = "c-api")]
use serde_json::{Value, json};
use shiro_rs::{c_api::*, definition::ModelDefinition, labels::PhoneMap, phonemap, segmentation};
use std::ptr::{null, null_mut};

#[test]
fn feature_frame_counts_preserve_failed_outputs() {
    // SAFETY: Independent exclusively writable output storage for each call.
    unsafe {
        let mut count = 99;
        assert_eq!(shiro_rs_feature_frame_count(144 * 12, 36, &mut count), 0);
        assert_eq!(count, 12);
        for (bytes, dimensions) in [
            (1, 36),
            (143, 36),
            (145, 36),
            (0, 0),
            (0, i32::MAX as usize + 1),
        ] {
            assert_eq!(
                shiro_rs_feature_frame_count(bytes, dimensions, &mut count),
                3
            );
            assert_eq!(count, 12);
        }
        assert_eq!(shiro_rs_feature_frame_count(0, 36, null_mut()), 1);
        assert_eq!(shiro_rs_feature_frame_count(0, 36, &mut count), 0);
        assert_eq!(count, 0);
    }
}

unsafe fn owned(values: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable input and independent initialized output slot.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(values.as_ptr(), values.len(), &mut output) },
        0
    );
    output
}
unsafe fn json_value(bytes: *const ShiroRsBytes) -> Value {
    // SAFETY: Live byte owner and independent output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(bytes, &mut count), 0);
        let mut values = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(bytes, 0, values.as_mut_ptr(), count), 0);
        serde_json::from_slice(&values).unwrap()
    }
}
fn assert_json(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => {
            assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= 1e-14)
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                assert_json(a, b);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (key, value) in a {
                assert_json(value, &b[key]);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

#[test]
fn original_lua_maps_definitions_and_initial_states_match_all_options() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/phones-original.json")).unwrap();
    // SAFETY: Unique owners, immutable borrowed inputs and independent outputs.
    unsafe {
        let mut input = owned(include_bytes!("fixtures/phones-input.txt"));
        let mut names = owned(br#"["bb","aa","bb","cc","aa"]"#);
        let mut config = ShiroRsPhoneOptions {
            states_per_phone: 0,
            streams: 0,
            weak_skips: 9,
        };
        assert_eq!(shiro_rs_phone_options_default(&mut config), 0);
        assert_eq!(
            config,
            ShiroRsPhoneOptions {
                states_per_phone: 3,
                streams: 3,
                weak_skips: 0
            }
        );
        for case in cases {
            config.states_per_phone = case["count"].as_u64().unwrap() as usize;
            config.streams = 2;
            config.weak_skips = 1;
            let mut topology = owned(case["topology"].as_str().unwrap().as_bytes());
            let mut map = null_mut();
            assert_eq!(
                shiro_rs_phone_map_create(input, &config, topology, &mut map),
                0
            );
            assert_eq!(shiro_rs_bytes_release(&mut topology), 0);
            let mut clone = null_mut();
            assert_eq!(shiro_rs_phone_map_clone(map, &mut clone), 0);
            assert_eq!(shiro_rs_phone_map_release(&mut map), 0);
            let mut bytes = null_mut();
            assert_eq!(shiro_rs_phone_map_write_json(clone, &mut bytes), 0);
            assert_json(&json_value(bytes), &case["map"]);
            assert_eq!(shiro_rs_phone_map_read_json(bytes, &mut map), 0);
            assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
            assert_eq!(shiro_rs_phone_map_release(&mut clone), 0);
            let mut states = null_mut();
            assert_eq!(
                shiro_rs_segmentation_initial(names, map, 41, &mut states),
                0
            );
            assert_eq!(shiro_rs_states_write_json(states, &mut bytes), 0);
            assert_json(
                &json_value(bytes),
                &case["segmentation"]["file_list"][0]["states"],
            );
            assert_eq!(shiro_rs_states_release(&mut states), 0);
            assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
            if !case["definition"].is_null() {
                assert_eq!(
                    shiro_rs_phone_map_to_definition(map, 12, 0.01, &mut bytes),
                    0
                );
                let actual: ModelDefinition = serde_json::from_value(json_value(bytes)).unwrap();
                let mut expected: ModelDefinition =
                    serde_json::from_value(case["definition"].clone()).unwrap();
                expected
                    .duration_constraints
                    .sort_by_key(|constraint| constraint.index);
                assert_json(
                    &serde_json::to_value(&actual).unwrap(),
                    &serde_json::to_value(expected).unwrap(),
                );
                let mut model = null_mut();
                assert_eq!(shiro_rs_model_from_definition(bytes, &mut model), 0);
                assert_eq!(shiro_rs_model_release(&mut model), 0);
                assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
            }
            assert_eq!(shiro_rs_phone_map_release(&mut map), 0);
        }
        assert_eq!(shiro_rs_bytes_release(&mut names), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
    }
}

#[test]
fn complete_metadata_optional_topology_and_failures_preserve_owners() {
    // SAFETY: Unique owners and independent initialized output slots.
    unsafe {
        let mut input = owned(b"aa\nbb\n");
        let config = ShiroRsPhoneOptions {
            states_per_phone: 3,
            streams: 2,
            weak_skips: 0,
        };
        for topology in [None, Some(""), Some("unknown"), Some("type-c")] {
            let mut encoded = topology
                .map(|value| owned(value.as_bytes()))
                .unwrap_or(null_mut());
            let mut map = null_mut();
            assert_eq!(
                shiro_rs_phone_map_create(input, &config, encoded, &mut map),
                0
            );
            let native = phonemap::create(
                "aa\nbb\n",
                &phonemap::Options {
                    states_per_phone: 3,
                    streams: 2,
                    topology: topology.map(str::to_owned),
                    weak_skips: false,
                },
            )
            .unwrap();
            let mut bytes = null_mut();
            assert_eq!(shiro_rs_phone_map_write_json(map, &mut bytes), 0);
            assert_eq!(json_value(bytes), serde_json::to_value(native).unwrap());
            assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
            assert_eq!(shiro_rs_phone_map_release(&mut map), 0);
            assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        }
        let value = json!({"phone_map":{"aa":{"states":[{"dur":0,"out":[0],"state_extra":{"x":[1,"retained"]}}],"durfloor":[0.005],"phone_extra":true}},"map_extra":{"nested":[null,3]}});
        let mut document = owned(&serde_json::to_vec(&value).unwrap());
        let mut map = null_mut();
        assert_eq!(shiro_rs_phone_map_read_json(document, &mut map), 0);
        assert_eq!(shiro_rs_bytes_release(&mut document), 0);
        let mut clone = null_mut();
        assert_eq!(shiro_rs_phone_map_clone(map, &mut clone), 0);
        assert_eq!(shiro_rs_phone_map_release(&mut map), 0);
        let mut bytes = null_mut();
        assert_eq!(shiro_rs_phone_map_write_json(clone, &mut bytes), 0);
        assert_eq!(json_value(bytes), value);
        let retained_bytes = bytes;
        assert_eq!(
            shiro_rs_phone_map_to_definition(clone, 0, 0.01, &mut bytes),
            3
        );
        assert_eq!(bytes, retained_bytes);
        let native: PhoneMap = serde_json::from_value(value).unwrap();
        let mut names = owned(br#"["aa"]"#);
        let mut states = null_mut();
        assert_eq!(
            shiro_rs_segmentation_initial(names, clone, 9, &mut states),
            0
        );
        let mut states_json = null_mut();
        assert_eq!(shiro_rs_states_write_json(states, &mut states_json), 0);
        assert_eq!(
            json_value(states_json),
            serde_json::to_value(segmentation::initial(&["aa".into()], &native, 9).unwrap())
                .unwrap()
        );
        let mut bad = owned(br#"["missing"]"#);
        let mut retained_states = states;
        assert_eq!(
            shiro_rs_segmentation_initial(bad, clone, 9, &mut retained_states),
            3
        );
        assert_eq!(retained_states, states);
        let mut retained_map = clone;
        assert_eq!(shiro_rs_phone_map_read_json(bad, &mut retained_map), 3);
        assert_eq!(retained_map, clone);
        let invalid = ShiroRsPhoneOptions {
            weak_skips: 2,
            ..config
        };
        assert_eq!(
            shiro_rs_phone_map_create(input, &invalid, null(), &mut retained_map),
            2
        );
        assert_eq!(retained_map, clone);
        assert_eq!(
            shiro_rs_phone_map_create(input, null(), null(), &mut retained_map),
            1
        );
        assert_eq!(shiro_rs_phone_options_default(null_mut()), 1);
        assert_eq!(shiro_rs_phone_map_clone(clone, null_mut()), 1);
        assert_eq!(shiro_rs_phone_map_release(null_mut()), 1);
        assert_eq!(shiro_rs_states_release(&mut states), 0);
        assert_eq!(shiro_rs_bytes_release(&mut states_json), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bad), 0);
        assert_eq!(shiro_rs_bytes_release(&mut names), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        assert_eq!(shiro_rs_phone_map_release(&mut clone), 0);
        assert_eq!(shiro_rs_phone_map_release(&mut clone), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
    }
}
