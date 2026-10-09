use shiro_rs_capi::*;
mod common;
use common::Directory;
use serde_json::{Value, json};
use shiro_rs::{
    alignment::{self, DurationMode, Options},
    dataset,
    hsmm::{GeometricOptions, HsmmOptions, Model},
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
unsafe fn states_json(states: *const ShiroRsStates) -> Value {
    // SAFETY: Live owner and independent initialized output slot.
    unsafe {
        let mut output = null_mut();
        assert_eq!(shiro_rs_states_write_json(states, &mut output), 0);
        let value = serde_json::from_slice(&copied(output)).unwrap();
        assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        value
    }
}
fn source(isolated: bool) -> SegmentationDocument {
    serde_json::from_slice(if isolated {
        include_bytes!("../../../tests/fixtures/align-c-isolated.json")
    } else {
        include_bytes!("../../../tests/fixtures/align-c-embedded.json")
    })
    .unwrap()
}
fn reference(index: usize) -> SegmentationDocument {
    let references: &[&[u8]] = &[
        include_bytes!("../../../tests/fixtures/align-c-embedded-hsmm.json"),
        include_bytes!("../../../tests/fixtures/align-c-embedded-hmm.json"),
        include_bytes!("../../../tests/fixtures/align-c-embedded-hsmm-pruned.json"),
        include_bytes!("../../../tests/fixtures/align-c-embedded-hmm-pruned.json"),
        include_bytes!("../../../tests/fixtures/align-c-isolated-hsmm.json"),
        include_bytes!("../../../tests/fixtures/align-c-isolated-hmm.json"),
        include_bytes!("../../../tests/fixtures/align-c-isolated-hsmm-pruned.json"),
        include_bytes!("../../../tests/fixtures/align-c-isolated-hmm-pruned.json"),
    ];
    serde_json::from_slice(references[index]).unwrap()
}
fn native_options(index: usize) -> Options {
    let mut options = Options {
        isolated: index >= 4,
        duration_mode: if index % 2 == 1 {
            DurationMode::Geometric
        } else {
            DurationMode::Explicit
        },
        ..Options::default()
    };
    options.geometric.pruning_slope = 0.8;
    if index % 4 >= 2 {
        options.hsmm.state_radius = 2.0;
        options.hsmm.duration_extra = 8;
        options.geometric.pruning_slope = 0.5;
    }
    options
}

#[test]
fn all_original_nine_inference_cases_and_complete_configuration_match() {
    let native_model =
        Model::read_from(include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm").as_slice())
            .unwrap();
    let native_observation = dataset::read_observation(
        include_bytes!("../../../tests/fixtures/init-input.bin").as_slice(),
        &[2, 1],
        12,
    )
    .unwrap();
    // SAFETY: Unique owners and independent initialized descriptor/output slots.
    unsafe {
        let mut model_bytes = owned(include_bytes!(
            "../../../tests/fixtures/init-c-aligned.hsmm"
        ));
        let mut model = null_mut();
        assert_eq!(
            shiro_rs_model_read_bytes(model_bytes, 16 * 1024 * 1024, &mut model),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut model_bytes), 0);
        let mut raw = owned(include_bytes!("../../../tests/fixtures/init-input.bin"));
        let mut observation = null_mut();
        assert_eq!(
            shiro_rs_observation_from_model_rawfloat(raw, model, 12, &mut observation),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut raw), 0);
        let mut defaults = Options::default().into();
        assert_eq!(shiro_rs_alignment_options_default(&mut defaults), 0);
        assert_eq!(defaults, ShiroRsAlignmentOptions::from(Options::default()));
        for index in 0..9 {
            let (document, options, expected) = if index == 8 {
                (
                    serde_json::from_slice::<SegmentationDocument>(include_bytes!(
                        "../../../tests/fixtures/align-c-four.json"
                    ))
                    .unwrap(),
                    Options {
                        duration_mode: DurationMode::Geometric,
                        ..Options::default()
                    },
                    serde_json::from_slice::<SegmentationDocument>(include_bytes!(
                        "../../../tests/fixtures/align-c-four-hmm.json"
                    ))
                    .unwrap(),
                )
            } else {
                (source(index >= 4), native_options(index), reference(index))
            };
            let original_json = serde_json::to_value(&document.files[0].states).unwrap();
            let mut input = owned(&serde_json::to_vec(&document.files[0].states).unwrap());
            let mut states = null_mut();
            let mut aligned = null_mut();
            assert_eq!(shiro_rs_states_read_json(input, &mut states), 0);
            assert_eq!(shiro_rs_bytes_release(&mut input), 0);
            let config = options.into();
            assert_eq!(
                shiro_rs_align_states(model, observation, states, &config, &mut aligned),
                0
            );
            assert_eq!(
                states_json(aligned),
                serde_json::to_value(&expected.files[0].states).unwrap(),
                "case {index}"
            );
            assert_eq!(states_json(states), original_json);
            assert_eq!(shiro_rs_states_release(&mut states), 0);
            assert_eq!(
                states_json(aligned),
                serde_json::to_value(
                    alignment::align_states(
                        &native_model,
                        &native_observation,
                        &document.files[0].states,
                        options
                    )
                    .unwrap()
                )
                .unwrap()
            );
            assert_eq!(shiro_rs_states_release(&mut aligned), 0);
        }
        for isolated in [false, true] {
            for geometric in [false, true] {
                let mut document = source(isolated);
                document.files[0].states[0]
                    .attributes
                    .insert("retained".into(), json!({"nested":[null,true,"keep"]}));
                let options = Options {
                    isolated,
                    duration_mode: if geometric {
                        DurationMode::Geometric
                    } else {
                        DurationMode::Explicit
                    },
                    hsmm: HsmmOptions {
                        temperature: 0.85,
                        duration_weight: 0.6,
                        state_radius: 20.0,
                        duration_extra: 40,
                        duration_extra_factor: 1.4,
                    },
                    geometric: GeometricOptions {
                        temperature: 0.9,
                        pruning_slope: 0.8,
                    },
                };
                // Construct independently instead of using the conversion under test.
                let config = ShiroRsAlignmentOptions {
                    duration_mode: u32::from(geometric),
                    isolated: u32::from(isolated),
                    hsmm_temperature: 0.85,
                    duration_weight: 0.6,
                    state_radius: 20.0,
                    duration_extra: 40,
                    duration_extra_factor: 1.4,
                    geometric_temperature: 0.9,
                    pruning_slope: 0.8,
                };
                let expected = alignment::align_states(
                    &native_model,
                    &native_observation,
                    &document.files[0].states,
                    options,
                )
                .unwrap();
                let mut input = owned(&serde_json::to_vec(&document.files[0].states).unwrap());
                let mut states = null_mut();
                let mut aligned = null_mut();
                assert_eq!(shiro_rs_states_read_json(input, &mut states), 0);
                assert_eq!(shiro_rs_bytes_release(&mut input), 0);
                assert_eq!(
                    shiro_rs_align_states(model, observation, states, &config, &mut aligned),
                    0
                );
                assert_eq!(
                    states_json(aligned),
                    serde_json::to_value(expected).unwrap()
                );
                assert_eq!(shiro_rs_states_release(&mut states), 0);
                assert_eq!(shiro_rs_states_release(&mut aligned), 0);
            }
        }
        assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}

#[test]
fn host_document_alignment_retains_all_metadata_and_file_semantics() {
    let directory = Directory::new("c-api-alignment");
    let path = directory.path().join("input.f");
    fs::write(
        &path,
        include_bytes!("../../../tests/fixtures/init-input.bin"),
    )
    .unwrap();
    let native_model =
        Model::read_from(include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm").as_slice())
            .unwrap();
    // SAFETY: Unique owners and independent initialized descriptors/output slots.
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
        for index in 0..8 {
            let mut document = source(index >= 4);
            document.files[0].filename = path.to_str().unwrap().into();
            document
                .attributes
                .insert("document_metadata".into(), json!({"keep":[1,null]}));
            document.files[0]
                .attributes
                .insert("file_metadata".into(), json!({"keep":true}));
            document.files[0].states[0]
                .metadata
                .push(json!({"extra":[false,"tail"]}));
            let options = native_options(index);
            let config = options.into();
            let expected = alignment::align_document(&native_model, &document, options).unwrap();
            let original = serde_json::to_vec(&document).unwrap();
            let mut input = owned(&original);
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_align_document(model, input, &config, &mut output),
                0
            );
            assert_eq!(copied(input), original);
            assert_eq!(
                serde_json::from_slice::<Value>(&copied(output)).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
            assert_eq!(shiro_rs_bytes_release(&mut input), 0);
            assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        }
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}

#[test]
fn invalid_options_paths_and_pointers_leave_outputs_and_inputs_intact() {
    // SAFETY: Unique owners and independent initialized retained output slots.
    unsafe {
        let mut source_bytes = owned(include_bytes!(
            "../../../tests/fixtures/init-c-aligned.hsmm"
        ));
        let mut model = null_mut();
        assert_eq!(
            shiro_rs_model_read_bytes(source_bytes, 16 * 1024 * 1024, &mut model),
            0
        );
        let mut raw = owned(include_bytes!("../../../tests/fixtures/init-input.bin"));
        let mut observation = null_mut();
        assert_eq!(
            shiro_rs_observation_from_model_rawfloat(raw, model, 12, &mut observation),
            0
        );
        let original = serde_json::to_vec(&source(false).files[0].states).unwrap();
        let mut input = owned(&original);
        let mut states = null_mut();
        assert_eq!(shiro_rs_states_read_json(input, &mut states), 0);
        let default: ShiroRsAlignmentOptions = Options::default().into();
        for config in [
            ShiroRsAlignmentOptions {
                duration_mode: 2,
                ..default
            },
            ShiroRsAlignmentOptions {
                isolated: 2,
                ..default
            },
        ] {
            let mut retained = states;
            assert_eq!(
                shiro_rs_align_states(model, observation, states, &config, &mut retained),
                2
            );
            assert_eq!(retained, states);
            let mut retained_bytes = input;
            assert_eq!(
                shiro_rs_align_document(model, input, &config, &mut retained_bytes),
                2
            );
            assert_eq!(retained_bytes, input);
        }
        let config = ShiroRsAlignmentOptions {
            hsmm_temperature: f32::NAN,
            ..default
        };
        let mut retained = states;
        assert_eq!(
            shiro_rs_align_states(model, observation, states, &config, &mut retained),
            3
        );
        assert_eq!(retained, states);
        assert_eq!(shiro_rs_alignment_options_default(null_mut()), 1);
        assert_eq!(
            shiro_rs_align_states(null(), observation, states, &default, &mut retained),
            1
        );
        assert_eq!(
            shiro_rs_align_states(model, null(), states, &default, &mut retained),
            1
        );
        assert_eq!(
            shiro_rs_align_states(model, observation, null(), &default, &mut retained),
            1
        );
        assert_eq!(
            shiro_rs_align_states(model, observation, states, null(), &mut retained),
            1
        );
        assert_eq!(
            shiro_rs_align_states(model, observation, states, &default, null_mut()),
            1
        );
        assert_eq!(retained, states);
        for document in [b"{}".as_slice(), b"{\"file_list\":[{\"filename\":\"/nonexistent-shiro-c-api-input-96cd017.f\",\"states\":[]}]}" ] {
            let mut bytes = owned(document); let mut retained_bytes = input;
            assert_eq!(shiro_rs_align_document(model, bytes, &default, &mut retained_bytes), 3); assert_eq!(retained_bytes, input);
            assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        }
        let mut retained_bytes = input;
        assert_eq!(
            shiro_rs_align_document(null(), input, &default, &mut retained_bytes),
            1
        );
        assert_eq!(
            shiro_rs_align_document(model, null(), &default, &mut retained_bytes),
            1
        );
        assert_eq!(
            shiro_rs_align_document(model, input, null(), &mut retained_bytes),
            1
        );
        assert_eq!(
            shiro_rs_align_document(model, input, &default, null_mut()),
            1
        );
        assert_eq!(retained_bytes, input);
        assert_eq!(copied(input), original);
        assert_eq!(
            states_json(states),
            serde_json::to_value(&source(false).files[0].states).unwrap()
        );
        let mut model_output = null_mut();
        assert_eq!(shiro_rs_model_write_bytes(model, 0, &mut model_output), 0);
        assert_eq!(copied(model_output), copied(source_bytes));
        assert_eq!(shiro_rs_bytes_release(&mut model_output), 0);
        assert_eq!(shiro_rs_states_release(&mut states), 0);
        assert_eq!(shiro_rs_observation_release(&mut observation), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert_eq!(shiro_rs_bytes_release(&mut source_bytes), 0);
        assert_eq!(shiro_rs_bytes_release(&mut raw), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
    }
}
