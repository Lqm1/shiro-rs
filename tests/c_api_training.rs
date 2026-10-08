#![cfg(feature = "c-api")]
mod common;
use common::Directory;
use shiro_rs::{
    c_api::*,
    dataset,
    hsmm::Model,
    labels::SegmentationDocument,
    training::{self, IterationReport, Options},
};
use std::{
    ffi::c_void,
    fs,
    ptr::{null, null_mut},
};

unsafe fn owned(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable input and independent initialized output slot.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn copied(bytes: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live readable owner and independent output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(bytes, &mut count), 0);
        let mut values = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(bytes, 0, values.as_mut_ptr(), count), 0);
        values
    }
}
unsafe fn model() -> *mut ShiroRsModel {
    // SAFETY: Unique input/output owners.
    unsafe {
        let mut bytes = owned(include_bytes!("fixtures/init-c-aligned.hsmm"));
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_model_read_bytes(bytes, 16 * 1024 * 1024, &mut output),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        output
    }
}
fn native_model() -> Model {
    Model::read_from(include_bytes!("fixtures/init-c-aligned.hsmm").as_slice()).unwrap()
}
fn document() -> SegmentationDocument {
    serde_json::from_slice(include_bytes!("fixtures/align-c-isolated.json")).unwrap()
}
unsafe fn report(report: *const ShiroRsIterationReport) -> IterationReport {
    // SAFETY: Live owned or callback-borrowed report and independent output buffers.
    unsafe {
        let mut info = ShiroRsIterationInfo {
            iteration: 99,
            temperature: 99.0,
            mean_log_likelihood: 99.0,
        };
        assert_eq!(shiro_rs_iteration_report_info(report, &mut info), 0);
        let mut count = 0;
        assert_eq!(shiro_rs_iteration_report_file_count(report, &mut count), 0);
        let mut rows = Vec::new();
        for index in 0..count {
            let mut row = null_mut();
            assert_eq!(
                shiro_rs_iteration_report_get_file(report, index, &mut row),
                0
            );
            let mut length = 0;
            assert_eq!(shiro_rs_array_f32_length(row, &mut length), 0);
            let mut values = vec![0.0; length];
            assert_eq!(
                shiro_rs_array_f32_copy(row, 0, values.as_mut_ptr(), length),
                0
            );
            assert_eq!(shiro_rs_array_f32_release(&mut row), 0);
            rows.push(values);
        }
        IterationReport {
            iteration: info.iteration,
            temperature: info.temperature,
            mean_log_likelihood: info.mean_log_likelihood,
            file_likelihoods: rows,
        }
    }
}
unsafe extern "C" fn progress(context: *mut c_void, value: *const ShiroRsIterationReport) {
    // SAFETY: Test context is a live exclusive Vec; report is callback-borrowed.
    unsafe {
        let mut clone = null_mut();
        assert_eq!(shiro_rs_iteration_report_clone(value, &mut clone), 0);
        (*(context.cast::<Vec<*mut ShiroRsIterationReport>>())).push(clone);
    }
}
unsafe fn model_wire(model: *const ShiroRsModel) -> Vec<u8> {
    // SAFETY: Live model and unique output owner.
    unsafe {
        let mut bytes = null_mut();
        assert_eq!(shiro_rs_model_write_bytes(model, 0, &mut bytes), 0);
        let result = copied(bytes);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        result
    }
}

#[test]
fn original_models_likelihoods_and_progress_reports_preserve_all_fields() {
    let directory = Directory::new("c-api-training");
    let path = directory.path().join("input.f");
    fs::write(&path, include_bytes!("fixtures/init-input.bin")).unwrap();
    let mut document = document();
    document.files[0].filename = path.to_str().unwrap().into();
    type ReferenceCase<'a> = (&'a [u8], &'a str, usize, u32, u32, u32, u32);
    let fixtures: &[ReferenceCase<'_>] = &[
        (
            include_bytes!("fixtures/rest-c-hsmm-one.hsmm"),
            include_str!("fixtures/rest-c-hsmm-one.likelihood"),
            1,
            0,
            0,
            0,
            0,
        ),
        (
            include_bytes!("fixtures/rest-c-hsmm-two.hsmm"),
            include_str!("fixtures/rest-c-hsmm-two.likelihood"),
            2,
            0,
            0,
            0,
            0,
        ),
        (
            include_bytes!("fixtures/rest-c-daem.hsmm"),
            include_str!("fixtures/rest-c-daem.likelihood"),
            3,
            0,
            1,
            0,
            0,
        ),
        (
            include_bytes!("fixtures/rest-c-hmm.hsmm"),
            include_str!("fixtures/rest-c-hmm.likelihood"),
            2,
            1,
            0,
            0,
            0,
        ),
        (
            include_bytes!("fixtures/rest-c-hsmm-one.hsmm"),
            include_str!("fixtures/rest-c-mean.likelihood"),
            1,
            0,
            0,
            1,
            0,
        ),
        (
            include_bytes!("fixtures/rest-c-isolated.hsmm"),
            include_str!("fixtures/rest-c-isolated.likelihood"),
            2,
            0,
            0,
            0,
            1,
        ),
        (
            include_bytes!("fixtures/rest-c-isolated-hmm.hsmm"),
            include_str!("fixtures/rest-c-isolated-hmm.likelihood"),
            2,
            1,
            0,
            0,
            1,
        ),
        (
            include_bytes!("fixtures/rest-c-isolated-daem.hsmm"),
            include_str!("fixtures/rest-c-isolated-daem.likelihood"),
            3,
            0,
            1,
            0,
            1,
        ),
        (
            include_bytes!("fixtures/rest-c-isolated-mean.hsmm"),
            include_str!("fixtures/rest-c-isolated-mean.likelihood"),
            2,
            0,
            0,
            1,
            1,
        ),
    ];
    // SAFETY: Unique owners and synchronous callbacks with valid exclusive contexts.
    unsafe {
        let mut model = model();
        let mut input = owned(&serde_json::to_vec(&document).unwrap());
        for &(expected_wire, likelihoods, iterations, mode, anneal, mean, isolated) in fixtures {
            let mut files = null_mut();
            assert_eq!(
                shiro_rs_training_files_read_document(model, input, 12, isolated, &mut files),
                0
            );
            let mut config = Options::default().into();
            assert_eq!(shiro_rs_training_options_default(&mut config), 0);
            config.iterations = iterations;
            config.duration_mode = mode;
            config.deterministic_annealing = anneal;
            config.mean_frame_likelihood = mean;
            config.termination_threshold = 0.0;
            config.pruning_slope = 0.8;
            let native_options = Options {
                iterations,
                duration_mode: if mode == 0 {
                    training::DurationMode::Normal
                } else {
                    training::DurationMode::Geometric
                },
                deterministic_annealing: anneal != 0,
                mean_frame_likelihood: mean != 0,
                termination_threshold: 0.0,
                geometric: shiro_rs::hsmm::GeometricOptions {
                    pruning_slope: 0.8,
                    ..Default::default()
                },
                ..Options::default()
            };
            let native_files =
                dataset::load_training_files(&document, &native_model(), 12, isolated != 0)
                    .unwrap();
            let native = training::train(&native_model(), &native_files, native_options).unwrap();
            let mut callbacks: Vec<*mut ShiroRsIterationReport> = Vec::new();
            let mut trained = null_mut();
            assert_eq!(
                shiro_rs_train_with_progress(
                    model,
                    files,
                    &config,
                    Some(progress),
                    (&mut callbacks as *mut Vec<*mut ShiroRsIterationReport>).cast(),
                    &mut trained
                ),
                0
            );
            let mut count = 0;
            assert_eq!(shiro_rs_training_result_length(trained, &mut count), 0);
            assert_eq!(count, iterations);
            assert_eq!(callbacks.len(), count);
            let mut clone = null_mut();
            assert_eq!(shiro_rs_training_result_clone(trained, &mut clone), 0);
            assert_eq!(shiro_rs_training_result_release(&mut trained), 0);
            let mut trained_model = null_mut();
            assert_eq!(
                shiro_rs_training_result_get_model(clone, &mut trained_model),
                0
            );
            assert_eq!(model_wire(trained_model), expected_wire);
            for (index, line) in likelihoods.lines().enumerate() {
                let mut value = null_mut();
                assert_eq!(
                    shiro_rs_training_result_get_report(clone, index, &mut value),
                    0
                );
                let actual = report(value);
                assert_eq!(actual, native.iterations[index]);
                assert_eq!(report(callbacks[index]), actual);
                let expected: Vec<f32> = line
                    .split(',')
                    .map(|value| value.parse().unwrap())
                    .collect();
                assert_eq!(actual.file_likelihoods[0].len(), expected.len());
                for (&actual, expected) in actual.file_likelihoods[0].iter().zip(expected) {
                    assert!((actual - expected).abs() <= 1e-5);
                }
                assert_eq!(shiro_rs_iteration_report_release(&mut value), 0);
            }
            assert_eq!(shiro_rs_training_result_release(&mut clone), 0);
            for mut value in callbacks {
                assert!(report(value).temperature.is_finite());
                assert_eq!(shiro_rs_iteration_report_release(&mut value), 0);
            }
            assert_eq!(shiro_rs_model_release(&mut trained_model), 0);
            assert_eq!(shiro_rs_training_files_release(&mut files), 0);
        }
        assert_eq!(
            model_wire(model),
            include_bytes!("fixtures/init-c-aligned.hsmm")
        );
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
    }
}

#[test]
fn file_and_group_snapshots_parallel_reduction_and_complete_config_match_native() {
    let directory = Directory::new("c-api-training-files");
    let path = directory.path().join("input.f");
    fs::write(&path, include_bytes!("fixtures/init-input.bin")).unwrap();
    let mut document = document();
    document.files[0].filename = path.to_str().unwrap().into();
    document.files.push(document.files[0].clone());
    let native_files = dataset::load_training_files(&document, &native_model(), 12, true).unwrap();
    // SAFETY: All owners unique and outputs independent initialized slots.
    unsafe {
        let mut model = model();
        let mut input = owned(&serde_json::to_vec(&document).unwrap());
        let mut files = null_mut();
        let mut clone = null_mut();
        assert_eq!(
            shiro_rs_training_files_read_document(model, input, 12, 1, &mut files),
            0
        );
        assert_eq!(shiro_rs_training_files_clone(files, &mut clone), 0);
        assert_eq!(shiro_rs_training_files_release(&mut files), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        let mut count = 0;
        assert_eq!(shiro_rs_training_files_length(clone, &mut count), 0);
        assert_eq!(count, 2);
        let mut snapshots = Vec::new();
        for (index, native_file) in native_files.iter().enumerate() {
            let mut data = null_mut();
            assert_eq!(
                shiro_rs_training_files_get_dataset(clone, index, &mut data),
                0
            );
            assert_eq!(shiro_rs_dataset_length(data, &mut count), 0);
            assert_eq!(count, 2);
            for group in 0..2 {
                let mut observation = null_mut();
                let mut bytes = null_mut();
                assert_eq!(
                    shiro_rs_dataset_get_observation(data, group, &mut observation),
                    0
                );
                assert_eq!(shiro_rs_observation_write_bytes(observation, &mut bytes), 0);
                let mut expected = Vec::new();
                native_file.observations[group]
                    .write_to(&mut expected)
                    .unwrap();
                assert_eq!(copied(bytes), expected);
                assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
                assert_eq!(shiro_rs_observation_release(&mut observation), 0);
                assert_eq!(
                    shiro_rs_dataset_get_segmentation_bytes(data, group, &mut bytes),
                    0
                );
                expected.clear();
                native_file.segmentations[group]
                    .write_to(&mut expected)
                    .unwrap();
                assert_eq!(copied(bytes), expected);
                assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
            }
            snapshots.push(data);
        }
        let mut constructed = null_mut();
        let pointers: Vec<_> = snapshots.iter().map(|p| p.cast_const()).collect();
        assert_eq!(
            shiro_rs_training_files_create(pointers.as_ptr(), pointers.len(), &mut constructed),
            0
        );
        assert_eq!(shiro_rs_training_files_release(&mut clone), 0);
        for mut snapshot in snapshots {
            assert_eq!(shiro_rs_dataset_release(&mut snapshot), 0);
        }
        let config = ShiroRsTrainingOptions {
            iterations: 2,
            duration_mode: 0,
            hsmm_temperature: 0.7,
            duration_weight: 0.8,
            state_radius: 20.0,
            duration_extra: 40,
            duration_extra_factor: 1.3,
            geometric_temperature: 0.9,
            pruning_slope: 0.8,
            termination_threshold: 0.0,
            deterministic_annealing: 0,
            mean_frame_likelihood: 1,
            workers: 2,
        };
        let options = Options {
            iterations: 2,
            hsmm: shiro_rs::hsmm::HsmmOptions {
                temperature: 0.7,
                duration_weight: 0.8,
                state_radius: 20.0,
                duration_extra: 40,
                duration_extra_factor: 1.3,
            },
            geometric: shiro_rs::hsmm::GeometricOptions {
                temperature: 0.9,
                pruning_slope: 0.8,
            },
            termination_threshold: 0.0,
            mean_frame_likelihood: true,
            workers: 2,
            ..Options::default()
        };
        let expected = training::train(&native_model(), &native_files, options).unwrap();
        let mut trained = null_mut();
        assert_eq!(shiro_rs_train(model, constructed, &config, &mut trained), 0);
        let mut output = null_mut();
        assert_eq!(shiro_rs_training_result_get_model(trained, &mut output), 0);
        let mut expected_wire = Vec::new();
        expected.model.write_to(&mut expected_wire).unwrap();
        assert_eq!(model_wire(output), expected_wire);
        for (index, expected) in expected.iterations.iter().enumerate() {
            let mut value = null_mut();
            assert_eq!(
                shiro_rs_training_result_get_report(trained, index, &mut value),
                0
            );
            assert_eq!(report(value), *expected);
            assert_eq!(shiro_rs_iteration_report_release(&mut value), 0);
        }
        assert_eq!(shiro_rs_model_release(&mut output), 0);
        assert_eq!(shiro_rs_training_result_release(&mut trained), 0);
        assert_eq!(shiro_rs_training_files_release(&mut constructed), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}

#[test]
fn zero_iterations_and_failures_retain_every_owner() {
    // SAFETY: Unique owners, readable pointer arrays and independent sentinel slots.
    unsafe {
        let mut model = model();
        let mut empty = null_mut();
        assert_eq!(shiro_rs_training_files_create(null(), 0, &mut empty), 0);
        let default: ShiroRsTrainingOptions = Options::default().into();
        let zero = ShiroRsTrainingOptions {
            iterations: 0,
            ..default
        };
        let mut trained = null_mut();
        let mut events: Vec<*mut ShiroRsIterationReport> = Vec::new();
        assert_eq!(
            shiro_rs_train_with_progress(
                model,
                empty,
                &zero,
                Some(progress),
                (&mut events as *mut Vec<*mut ShiroRsIterationReport>).cast(),
                &mut trained
            ),
            0
        );
        assert!(events.is_empty());
        let mut count = 99;
        assert_eq!(shiro_rs_training_result_length(trained, &mut count), 0);
        assert_eq!(count, 0);
        let mut output = null_mut();
        assert_eq!(shiro_rs_training_result_get_model(trained, &mut output), 0);
        assert_eq!(model_wire(output), model_wire(model));
        assert_eq!(shiro_rs_model_release(&mut output), 0);
        for invalid in [
            ShiroRsTrainingOptions {
                duration_mode: 2,
                ..zero
            },
            ShiroRsTrainingOptions {
                deterministic_annealing: 2,
                ..zero
            },
            ShiroRsTrainingOptions {
                mean_frame_likelihood: 2,
                ..zero
            },
            ShiroRsTrainingOptions { workers: 0, ..zero },
        ] {
            let mut retained = trained;
            assert_eq!(
                shiro_rs_train(model, empty, &invalid, &mut retained),
                if invalid.workers == 0 { 3 } else { 2 }
            );
            assert_eq!(retained, trained);
        }
        let mut retained = trained;
        assert_eq!(shiro_rs_train(model, empty, &default, &mut retained), 3);
        assert_eq!(retained, trained);
        let mut retained_files = empty;
        assert_eq!(
            shiro_rs_training_files_create(null(), usize::MAX, &mut retained_files),
            2
        );
        assert_eq!(retained_files, empty);
        assert_eq!(
            shiro_rs_training_files_create([null()].as_ptr(), 1, &mut retained_files),
            1
        );
        assert_eq!(retained_files, empty);
        let mut file = null_mut();
        assert_eq!(shiro_rs_training_files_get_dataset(empty, 0, &mut file), 2);
        assert!(file.is_null());
        let mut value = null_mut();
        assert_eq!(
            shiro_rs_training_result_get_report(trained, 0, &mut value),
            2
        );
        assert!(value.is_null());
        assert_eq!(shiro_rs_training_options_default(null_mut()), 1);
        assert_eq!(shiro_rs_train(model, empty, null(), &mut retained), 1);
        assert_eq!(shiro_rs_train(model, empty, &zero, null_mut()), 1);
        assert_eq!(shiro_rs_training_files_release(null_mut()), 1);
        assert_eq!(shiro_rs_training_result_release(null_mut()), 1);
        assert_eq!(shiro_rs_iteration_report_release(null_mut()), 1);
        assert_eq!(shiro_rs_training_result_release(&mut trained), 0);
        assert_eq!(shiro_rs_training_files_release(&mut empty), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}

#[test]
fn stopping_and_failed_estimation_preserve_callback_and_output_contracts() {
    let directory = Directory::new("c-api-training-stopping");
    let path = directory.path().join("input.f");
    fs::write(&path, include_bytes!("fixtures/init-input.bin")).unwrap();
    let mut document = document();
    document.files[0].filename = path.to_str().unwrap().into();
    document.files.push(document.files[0].clone());
    let native_files = dataset::load_training_files(&document, &native_model(), 12, true).unwrap();
    // SAFETY: Live immutable inputs, exclusive callback context and independent outputs.
    unsafe {
        let mut model = model();
        let original_model = model_wire(model);
        let mut input = owned(&serde_json::to_vec(&document).unwrap());
        let mut files = null_mut();
        assert_eq!(
            shiro_rs_training_files_read_document(model, input, 12, 1, &mut files),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        for mode in 0..2 {
            let options = Options {
                iterations: 5,
                duration_mode: if mode == 0 {
                    training::DurationMode::Normal
                } else {
                    training::DurationMode::Geometric
                },
                termination_threshold: f32::MAX,
                workers: 2,
                ..Options::default()
            };
            let expected = training::train(&native_model(), &native_files, options).unwrap();
            assert_eq!(expected.iterations.len(), 2);
            let config: ShiroRsTrainingOptions = options.into();
            let mut result = null_mut();
            let mut callbacks: Vec<*mut ShiroRsIterationReport> = Vec::new();
            assert_eq!(
                shiro_rs_train_with_progress(
                    model,
                    files,
                    &config,
                    Some(progress),
                    (&mut callbacks as *mut Vec<*mut ShiroRsIterationReport>).cast(),
                    &mut result
                ),
                0
            );
            let mut count = 99;
            assert_eq!(shiro_rs_training_result_length(result, &mut count), 0);
            assert_eq!(count, 2);
            assert_eq!(callbacks.len(), count);
            assert_eq!(shiro_rs_training_result_release(&mut result), 0);
            for (index, mut value) in callbacks.into_iter().enumerate() {
                assert_eq!(report(value), expected.iterations[index]);
                let mut row = null_mut();
                assert_eq!(shiro_rs_iteration_report_get_file(value, 2, &mut row), 2);
                assert!(row.is_null());
                assert_eq!(shiro_rs_iteration_report_release(&mut value), 0);
            }
            let mut retained = null_mut();
            let zero = ShiroRsTrainingOptions {
                iterations: 0,
                ..config
            };
            assert_eq!(shiro_rs_train(model, files, &zero, &mut retained), 0);
            let owner = retained;
            let invalid = ShiroRsTrainingOptions {
                duration_weight: f32::NAN,
                pruning_slope: f32::NAN,
                ..config
            };
            let mut events: Vec<*mut ShiroRsIterationReport> = Vec::new();
            assert_eq!(
                shiro_rs_train_with_progress(
                    model,
                    files,
                    &invalid,
                    Some(progress),
                    (&mut events as *mut Vec<*mut ShiroRsIterationReport>).cast(),
                    &mut retained
                ),
                3
            );
            assert_eq!(retained, owner);
            assert!(events.is_empty());
            assert_eq!(model_wire(model), original_model);
            assert_eq!(shiro_rs_training_result_release(&mut retained), 0);
        }
        assert_eq!(shiro_rs_training_files_release(&mut files), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}
