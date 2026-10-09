use shiro_rs_capi::*;
mod common;
use common::Directory;
use shiro_rs::{
    dataset,
    definition::ModelDefinition,
    hsmm::{Dataset, Model, Observation, Segmentation},
    initialization::{self, Options},
    labels::SegmentationDocument,
};
use std::{
    fs,
    ptr::{null, null_mut},
};

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
    // SAFETY: Live owner and independent initialized output buffers.
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
unsafe fn model() -> *mut ShiroRsModel {
    // SAFETY: Unique input owner and independent initialized output slot.
    unsafe {
        let mut definition = owned(include_bytes!(
            "../../../tests/fixtures/init-definition.json"
        ));
        let mut output = null_mut();
        assert_eq!(shiro_rs_model_from_definition(definition, &mut output), 0);
        assert_eq!(shiro_rs_bytes_release(&mut definition), 0);
        output
    }
}
fn native_model() -> Model {
    serde_json::from_slice::<ModelDefinition>(include_bytes!(
        "../../../tests/fixtures/init-definition.json"
    ))
    .unwrap()
    .build()
    .unwrap()
}
fn document() -> SegmentationDocument {
    serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/init-segmentation.json"
    ))
    .unwrap()
}
unsafe fn sample(
    model: *const ShiroRsModel,
    raw: &[u8],
    states: &[shiro_rs::labels::State],
) -> (*mut ShiroRsObservation, *mut ShiroRsStates) {
    // SAFETY: Live model and unique input/output owners.
    unsafe {
        let mut bytes = owned(raw);
        let mut observation = null_mut();
        let mut sequence = null_mut();
        assert_eq!(
            shiro_rs_observation_from_model_rawfloat(bytes, model, 12, &mut observation),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        bytes = owned(&serde_json::to_vec(states).unwrap());
        assert_eq!(shiro_rs_states_read_json(bytes, &mut sequence), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        (observation, sequence)
    }
}

#[test]
fn original_four_initializers_and_ten_frame_rounding_match_exact_c_models() {
    let cases: &[(u32, u32, f32, &[u8])] = &[
        (
            0,
            0,
            0.1,
            include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm"),
        ),
        (
            1,
            0,
            0.1,
            include_bytes!("../../../tests/fixtures/init-c-flat.hsmm"),
        ),
        (
            0,
            1,
            0.1,
            include_bytes!("../../../tests/fixtures/init-c-tied.hsmm"),
        ),
        (
            1,
            1,
            0.4,
            include_bytes!("../../../tests/fixtures/init-c-flat-tied.hsmm"),
        ),
    ];
    // SAFETY: Unique owners and independent initialized output slots.
    unsafe {
        let mut model = model();
        let states = document().files.remove(0).states;
        let (mut observation, mut sequence) = sample(
            model,
            include_bytes!("../../../tests/fixtures/init-input.bin"),
            &states,
        );
        let mut data = null_mut();
        let mut cloned = null_mut();
        assert_eq!(
            shiro_rs_dataset_create(
                model,
                [observation.cast_const()].as_ptr(),
                [sequence.cast_const()].as_ptr(),
                1,
                &mut data
            ),
            0
        );
        assert_eq!(shiro_rs_dataset_clone(data, &mut cloned), 0);
        assert_eq!(shiro_rs_dataset_release(&mut data), 0);
        assert!(data.is_null());
        assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        assert_eq!(shiro_rs_states_release(&mut sequence), 0);
        let mut defaults = ShiroRsInitializationOptions {
            flat_start: 99,
            globally_tied: 99,
            variance_floor_ratio: 99.0,
        };
        assert_eq!(shiro_rs_initialization_options_default(&mut defaults), 0);
        assert_eq!(
            defaults,
            ShiroRsInitializationOptions {
                flat_start: 0,
                globally_tied: 0,
                variance_floor_ratio: 0.1
            }
        );
        for &(flat_start, globally_tied, variance_floor_ratio, expected) in cases {
            let config = ShiroRsInitializationOptions {
                flat_start,
                globally_tied,
                variance_floor_ratio,
            };
            let mut initialized = null_mut();
            let mut encoded = null_mut();
            assert_eq!(
                shiro_rs_initialize(model, cloned, &config, &mut initialized),
                0
            );
            assert_eq!(shiro_rs_model_write_bytes(initialized, 0, &mut encoded), 0);
            assert_eq!(copied(encoded), expected);
            let mut reloaded = null_mut();
            assert_eq!(
                shiro_rs_model_read_bytes(encoded, 16 * 1024 * 1024, &mut reloaded),
                0
            );
            assert_eq!(shiro_rs_model_release(&mut initialized), 0);
            assert_eq!(shiro_rs_model_release(&mut reloaded), 0);
            assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        }
        let mut length = 99;
        assert_eq!(shiro_rs_dataset_length(cloned, &mut length), 0);
        assert_eq!(length, 1);
        let mut snapshot = null_mut();
        let mut seg_bytes = null_mut();
        assert_eq!(
            shiro_rs_dataset_get_observation(cloned, 0, &mut snapshot),
            0
        );
        assert_eq!(
            shiro_rs_dataset_get_segmentation_bytes(cloned, 0, &mut seg_bytes),
            0
        );
        assert_eq!(shiro_rs_dataset_release(&mut cloned), 0);
        let expected_seg = dataset::read_segmentation(&states, &native_model()).unwrap();
        assert_eq!(
            Segmentation::read_from(copied(seg_bytes).as_slice()).unwrap(),
            expected_seg
        );
        assert_eq!(shiro_rs_bytes_release(&mut seg_bytes), 0);
        assert_eq!(shiro_rs_observation_release(&mut snapshot), 0);
        let (mut ten, mut sequence) = sample(
            model,
            &include_bytes!("../../../tests/fixtures/init-input.bin")[..120],
            &states,
        );
        assert_eq!(
            shiro_rs_dataset_create(
                model,
                [ten.cast_const()].as_ptr(),
                [sequence.cast_const()].as_ptr(),
                1,
                &mut data
            ),
            0
        );
        let config = ShiroRsInitializationOptions {
            flat_start: 1,
            ..defaults
        };
        let mut initialized = null_mut();
        let mut encoded = null_mut();
        assert_eq!(
            shiro_rs_initialize(model, data, &config, &mut initialized),
            0
        );
        assert_eq!(shiro_rs_model_write_bytes(initialized, 0, &mut encoded), 0);
        assert_eq!(
            copied(encoded),
            include_bytes!("../../../tests/fixtures/init-c-flat-ten.hsmm")
        );
        assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        assert_eq!(shiro_rs_model_release(&mut initialized), 0);
        assert_eq!(shiro_rs_dataset_release(&mut data), 0);
        assert_eq!(shiro_rs_observation_release(&mut ten), 0);
        assert_eq!(shiro_rs_states_release(&mut sequence), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}

#[test]
fn multiple_host_samples_and_independent_snapshots_retain_complete_fields() {
    let directory = Directory::new("c-api-initialization");
    let feature = directory.path().join("input.f");
    fs::write(
        &feature,
        include_bytes!("../../../tests/fixtures/init-input.bin"),
    )
    .unwrap();
    let mut document = document();
    document.files[0].filename = feature.to_str().unwrap().into();
    document.files.push(document.files[0].clone());
    document.files[1].states[2].time = 999.0;
    let native = native_model();
    let expected = dataset::load(&document, &native, 12).unwrap();
    let expected_model =
        initialization::initialize(&native, &expected, Options::default()).unwrap();
    assert_eq!(expected_model.durations[3].mean, 4.0);
    assert_eq!(expected_model.durations[3].variance, 16.0);
    // SAFETY: Unique owners and independent initialized output slots and buffers.
    unsafe {
        let mut model = model();
        let original = serde_json::to_vec(&document).unwrap();
        let mut bytes = owned(&original);
        let mut data = null_mut();
        assert_eq!(
            shiro_rs_dataset_read_document(model, bytes, 12, &mut data),
            0
        );
        assert_eq!(copied(bytes), original);
        let mut length = 0;
        assert_eq!(shiro_rs_dataset_length(data, &mut length), 0);
        assert_eq!(length, 2);
        for index in 0..2 {
            let mut observation = null_mut();
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_dataset_get_observation(data, index, &mut observation),
                0
            );
            assert_eq!(
                shiro_rs_observation_write_bytes(observation, &mut output),
                0
            );
            assert_eq!(
                Observation::read_from(copied(output).as_slice()).unwrap(),
                expected.observations[index]
            );
            assert_eq!(shiro_rs_bytes_release(&mut output), 0);
            assert_eq!(shiro_rs_observation_release(&mut observation), 0);
            assert_eq!(
                shiro_rs_dataset_get_segmentation_bytes(data, index, &mut output),
                0
            );
            assert_eq!(
                Segmentation::read_from(copied(output).as_slice()).unwrap(),
                expected.segmentations[index]
            );
            assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        }
        let config: ShiroRsInitializationOptions = Options::default().into();
        let mut initialized = null_mut();
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_initialize(model, data, &config, &mut initialized),
            0
        );
        assert_eq!(shiro_rs_model_write_bytes(initialized, 0, &mut output), 0);
        let mut expected_wire = Vec::new();
        expected_model.write_to(&mut expected_wire).unwrap();
        assert_eq!(copied(output), expected_wire);
        let (mut observation, mut sequence) = sample(
            model,
            include_bytes!("../../../tests/fixtures/init-input.bin"),
            &document.files[0].states,
        );
        let mut repeated = null_mut();
        assert_eq!(
            shiro_rs_dataset_create(
                model,
                [observation.cast_const(), observation.cast_const()].as_ptr(),
                [sequence.cast_const(), sequence.cast_const()].as_ptr(),
                2,
                &mut repeated
            ),
            0
        );
        assert_eq!(shiro_rs_dataset_length(repeated, &mut length), 0);
        assert_eq!(length, 2);
        assert_eq!(shiro_rs_dataset_release(&mut repeated), 0);
        assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        assert_eq!(shiro_rs_states_release(&mut sequence), 0);
        assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        assert_eq!(shiro_rs_model_release(&mut initialized), 0);
        assert_eq!(shiro_rs_dataset_release(&mut data), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}

#[test]
fn empty_late_invalid_samples_and_bad_options_retain_outputs() {
    // SAFETY: Unique owners, readable arrays and independent initialized output slots.
    unsafe {
        let mut model = model();
        let states = document().files.remove(0).states;
        let (mut observation, mut sequence) = sample(
            model,
            include_bytes!("../../../tests/fixtures/init-input.bin"),
            &states,
        );
        let mut data = null_mut();
        assert_eq!(
            shiro_rs_dataset_create(
                model,
                [observation.cast_const()].as_ptr(),
                [sequence.cast_const()].as_ptr(),
                1,
                &mut data
            ),
            0
        );
        let config: ShiroRsInitializationOptions = Options::default().into();
        for invalid in [
            ShiroRsInitializationOptions {
                flat_start: 2,
                ..config
            },
            ShiroRsInitializationOptions {
                globally_tied: 2,
                ..config
            },
            ShiroRsInitializationOptions {
                variance_floor_ratio: -1.0,
                ..config
            },
            ShiroRsInitializationOptions {
                variance_floor_ratio: f32::NAN,
                ..config
            },
        ] {
            let mut retained = model;
            assert_eq!(
                shiro_rs_initialize(model, data, &invalid, &mut retained),
                if invalid.flat_start == 2 || invalid.globally_tied == 2 {
                    2
                } else {
                    3
                }
            );
            assert_eq!(retained, model);
        }
        let mut bytes = owned(b"[{}]");
        let mut invalid_states = null_mut();
        assert_eq!(shiro_rs_states_read_json(bytes, &mut invalid_states), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        let mut retained = data;
        assert_eq!(
            shiro_rs_dataset_create(
                model,
                [observation.cast_const(), observation.cast_const()].as_ptr(),
                [sequence.cast_const(), invalid_states.cast_const()].as_ptr(),
                2,
                &mut retained
            ),
            3
        );
        assert_eq!(retained, data);
        assert_eq!(
            shiro_rs_dataset_create(
                model,
                [observation.cast_const(), null()].as_ptr(),
                [sequence.cast_const(), sequence.cast_const()].as_ptr(),
                2,
                &mut retained
            ),
            1
        );
        assert_eq!(retained, data);
        assert_eq!(
            shiro_rs_dataset_create(model, null(), null(), usize::MAX, &mut retained),
            2
        );
        assert_eq!(retained, data);
        let mut snapshot = observation;
        assert_eq!(shiro_rs_dataset_get_observation(data, 1, &mut snapshot), 2);
        assert_eq!(snapshot, observation);
        bytes = owned(b"keep");
        let mut retained_bytes = bytes;
        assert_eq!(
            shiro_rs_dataset_get_segmentation_bytes(data, 1, &mut retained_bytes),
            2
        );
        assert_eq!(retained_bytes, bytes);
        let mut empty = null_mut();
        assert_eq!(
            shiro_rs_dataset_create(model, null(), null(), 0, &mut empty),
            0
        );
        let mut count = 99;
        assert_eq!(shiro_rs_dataset_length(empty, &mut count), 0);
        assert_eq!(count, 0);
        let mut retained_model = model;
        assert_eq!(
            shiro_rs_initialize(model, empty, &config, &mut retained_model),
            3
        );
        assert_eq!(retained_model, model);
        assert_eq!(shiro_rs_initialization_options_default(null_mut()), 1);
        assert_eq!(
            shiro_rs_initialize(model, data, null(), &mut retained_model),
            1
        );
        assert_eq!(shiro_rs_initialize(model, data, &config, null_mut()), 1);
        assert_eq!(shiro_rs_dataset_length(null(), &mut count), 1);
        assert_eq!(count, 0);
        assert_eq!(shiro_rs_dataset_clone(data, null_mut()), 1);
        assert_eq!(shiro_rs_dataset_release(null_mut()), 1);
        let mut malformed = owned(b"{}");
        assert_eq!(
            shiro_rs_dataset_read_document(model, malformed, 12, &mut retained),
            3
        );
        assert_eq!(retained, data);
        assert_eq!(shiro_rs_bytes_release(&mut malformed), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        assert_eq!(shiro_rs_dataset_release(&mut empty), 0);
        assert_eq!(shiro_rs_dataset_release(&mut empty), 0);
        assert_eq!(shiro_rs_dataset_release(&mut data), 0);
        assert_eq!(shiro_rs_states_release(&mut invalid_states), 0);
        assert_eq!(shiro_rs_states_release(&mut sequence), 0);
        assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
    // Native empty initialization rejection is retained rather than bypassed.
    assert!(
        initialization::initialize(&native_model(), &Dataset::default(), Options::default())
            .is_err()
    );
}
