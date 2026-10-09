use liblrhsmm_rs::{Dataset, Model};
use shiro_rs::{
    dataset,
    labels::SegmentationDocument,
    training::{self, DurationMode, Options},
};

fn model() -> Model {
    Model::read_from(include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm").as_slice())
        .unwrap()
}
fn file() -> Dataset {
    let model = model();
    let document: SegmentationDocument = serde_json::from_str(include_str!(
        "../../../tests/fixtures/align-c-isolated.json"
    ))
    .unwrap();
    Dataset {
        observations: vec![
            dataset::read_observation(
                include_bytes!("../../../tests/fixtures/init-input.bin").as_slice(),
                &dataset::dimensions(&model).unwrap(),
                12,
            )
            .unwrap(),
        ],
        segmentations: vec![dataset::read_segmentation(&document.files[0].states, &model).unwrap()],
    }
}

#[test]
fn embedded_iterations_and_daem_match_corrected_c() {
    let source = model();
    let input = file();
    for (options, expected) in [
        (
            Options::default(),
            include_bytes!("../../../tests/fixtures/rest-c-hsmm-one.hsmm").as_slice(),
        ),
        (
            Options {
                iterations: 2,
                termination_threshold: 0.0,
                ..Options::default()
            },
            include_bytes!("../../../tests/fixtures/rest-c-hsmm-two.hsmm").as_slice(),
        ),
        (
            Options {
                iterations: 3,
                termination_threshold: 0.0,
                deterministic_annealing: true,
                ..Options::default()
            },
            include_bytes!("../../../tests/fixtures/rest-c-daem.hsmm").as_slice(),
        ),
        (
            {
                let mut options = Options {
                    iterations: 2,
                    duration_mode: DurationMode::Geometric,
                    termination_threshold: 0.0,
                    ..Options::default()
                };
                options.geometric.pruning_slope = 0.8;
                options
            },
            include_bytes!("../../../tests/fixtures/rest-c-hmm.hsmm").as_slice(),
        ),
    ] {
        let trained = training::train(&source, std::slice::from_ref(&input), options).unwrap();
        assert_eq!(trained.iterations.len(), options.iterations);
        let likelihoods = if options.duration_mode == DurationMode::Geometric {
            include_str!("../../../tests/fixtures/rest-c-hmm.likelihood")
        } else if options.deterministic_annealing {
            include_str!("../../../tests/fixtures/rest-c-daem.likelihood")
        } else if options.iterations == 2 {
            include_str!("../../../tests/fixtures/rest-c-hsmm-two.likelihood")
        } else {
            include_str!("../../../tests/fixtures/rest-c-hsmm-one.likelihood")
        };
        for (report, expected) in trained.iterations.iter().zip(likelihoods.lines()) {
            let expected: f32 = expected.parse().unwrap();
            assert!((report.file_likelihoods[0][0] - expected).abs() <= 1e-5);
        }
        let expected_model = Model::read_from(expected).unwrap();
        assert_eq!(trained.model, expected_model, "{options:?}");
        let mut encoded = Vec::new();
        trained.model.write_to(&mut encoded).unwrap();
        assert_eq!(encoded, expected);
    }
    assert_eq!(source, model());
    assert_eq!(input, file());
}

#[test]
fn isolated_group_statistics_and_likelihood_rows_match_corrected_c() {
    let input = file();
    let dimensions = dataset::dimensions(&model()).unwrap();
    let mut groups = Dataset::default();
    for group in 0..2 {
        let mut observation = liblrhsmm_rs::Observation::new(6, &dimensions).unwrap();
        for (source, target) in input.observations[0]
            .streams
            .iter()
            .zip(&mut observation.streams)
        {
            let start = group * 6 * source.dimensions;
            target
                .values
                .copy_from_slice(&source.values[start..start + 6 * source.dimensions]);
        }
        let source = &input.segmentations[0];
        let mut segmentation = liblrhsmm_rs::Segmentation::new(2, 3);
        for state in 0..3 {
            segmentation.boundaries[state] =
                source.boundaries[group * 3 + state] - (group * 6) as i32;
            segmentation.duration_states[state] = source.duration_states[group * 3 + state];
            for stream in 0..2 {
                segmentation.output_states[stream][state] =
                    source.output_states[stream][group * 3 + state];
            }
        }
        groups.observations.push(observation);
        groups.segmentations.push(segmentation);
    }
    let result = training::train(
        &model(),
        &[groups],
        Options {
            iterations: 2,
            termination_threshold: 0.0,
            ..Options::default()
        },
    )
    .unwrap();
    let mut bytes = Vec::new();
    result.model.write_to(&mut bytes).unwrap();
    assert_eq!(
        bytes,
        include_bytes!("../../../tests/fixtures/rest-c-isolated.hsmm")
    );
    for (report, row) in result
        .iterations
        .iter()
        .zip(include_str!("../../../tests/fixtures/rest-c-isolated.likelihood").lines())
    {
        let expected: Vec<f32> = row.split(',').map(|value| value.parse().unwrap()).collect();
        assert_eq!(report.file_likelihoods[0].len(), expected.len());
        for (&actual, expected) in report.file_likelihoods[0].iter().zip(expected) {
            assert!((actual - expected).abs() <= 1e-5, "{actual} / {expected}");
        }
    }
}

#[test]
fn likelihood_reporting_convergence_and_zero_iterations_follow_source() {
    let source = model();
    let input = file();
    let mut progress = Vec::new();
    let trained = training::train_with_progress(
        &source,
        std::slice::from_ref(&input),
        Options {
            iterations: 5,
            ..Options::default()
        },
        |report| progress.push(report.clone()),
    )
    .unwrap();
    assert_eq!(trained.iterations.len(), 2);
    assert_eq!(trained.iterations, progress);
    assert!((trained.iterations[0].mean_log_likelihood - -36.458187).abs() < 0.00001);
    assert!((trained.iterations[1].mean_log_likelihood - -37.164482).abs() < 0.00001);
    let mean = training::train(
        &source,
        &[input],
        Options {
            mean_frame_likelihood: true,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(
        mean.model,
        training::train(&source, &[file()], Options::default())
            .unwrap()
            .model
    );
    assert_eq!(
        mean.iterations[0].mean_log_likelihood,
        progress[0].mean_log_likelihood / 12.0
    );
    let expected: f32 = include_str!("../../../tests/fixtures/rest-c-mean.likelihood")
        .trim()
        .parse()
        .unwrap();
    assert!((mean.iterations[0].file_likelihoods[0][0] - expected).abs() <= 1e-6);
    let zero = training::train(
        &source,
        &[],
        Options {
            iterations: 0,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(zero.model, source);
    assert!(zero.iterations.is_empty());
}

#[test]
fn parallel_reduction_is_repeatable_and_worker_errors_propagate() {
    let source = model();
    let files = vec![file(); 4];
    let options = Options {
        iterations: 2,
        termination_threshold: 0.0,
        workers: 3,
        ..Options::default()
    };
    let parallel = training::train(&source, &files, options).unwrap();
    let repeated = training::train(&source, &files, options).unwrap();
    assert_eq!(parallel.model, repeated.model);
    assert_eq!(parallel.iterations, repeated.iterations);
    let serial = training::train(
        &source,
        &files,
        Options {
            workers: 1,
            ..options
        },
    )
    .unwrap();
    assert_eq!(parallel.model.durations, serial.model.durations);
    let mut worst = 0.0f32;
    for (actual, expected) in parallel.model.streams.iter().zip(&serial.model.streams) {
        assert_eq!(actual.weight, expected.weight);
        for (actual, expected) in actual.mixtures.iter().zip(&expected.mixtures) {
            assert_eq!(actual.dimensions, expected.dimensions);
            let actual = actual
                .weights
                .iter()
                .chain(&actual.means)
                .chain(&actual.variances)
                .chain(&actual.variance_floors);
            let expected = expected
                .weights
                .iter()
                .chain(&expected.means)
                .chain(&expected.variances)
                .chain(&expected.variance_floors);
            for (&actual, &expected) in actual.zip(expected) {
                worst = worst.max((actual - expected).abs() / (1.0 + expected.abs()));
            }
        }
    }
    println!("parallel/serial maximum normalized parameter difference: {worst:e}");
    assert!(worst <= 2e-6);
    for (actual, expected) in parallel.iterations.iter().zip(&serial.iterations) {
        let difference = (actual.mean_log_likelihood - expected.mean_log_likelihood).abs()
            / (1.0 + expected.mean_log_likelihood.abs());
        println!(
            "iteration {} normalized likelihood difference: {difference:e}",
            actual.iteration
        );
        assert!(difference <= 2e-6);
    }
    let mut invalid = files;
    invalid[1].segmentations[0].output_states[0][0] = 999;
    assert!(training::train(&source, &invalid, options).is_err());
    assert_eq!(source, model());
}

#[test]
fn invalid_files_and_settings_leave_inputs_unchanged() {
    let source = model();
    assert!(training::train(&source, &[], Options::default()).is_err());
    assert!(training::train(&source, &[Dataset::default()], Options::default()).is_err());
    assert!(
        training::train(
            &source,
            &[file()],
            Options {
                workers: 0,
                ..Options::default()
            }
        )
        .is_err()
    );
    assert!(
        training::train(
            &source,
            &[file()],
            Options {
                termination_threshold: f32::NAN,
                ..Options::default()
            }
        )
        .is_err()
    );
    let mut input = file();
    input.segmentations[0].boundaries[5] = 999;
    let clamped =
        training::train(&source, std::slice::from_ref(&input), Options::default()).unwrap();
    assert_eq!(
        clamped.model,
        training::train(&source, &[file()], Options::default())
            .unwrap()
            .model
    );
    assert_eq!(input.segmentations[0].boundaries[5], 999);
    input.observations[0].frames = 0;
    assert!(training::train(&source, &[input], Options::default()).is_err());
}
