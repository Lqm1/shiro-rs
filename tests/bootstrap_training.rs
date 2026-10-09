mod common;

use common::Directory;
use liblrhsmm_rs::{Dataset, Model};
use shiro_rs::{
    alignment, dataset,
    definition::ModelDefinition,
    initialization,
    labels::SegmentationDocument,
    training::{self, DurationMode},
};
use std::fs;

fn saved_model(model: &Model, directory: &Directory, name: &str, expected: &[u8]) -> Model {
    let path = directory.path().join(name);
    model
        .write_to(&mut fs::File::create(&path).unwrap())
        .unwrap();
    assert_eq!(fs::read(&path).unwrap(), expected);
    let reloaded = Model::read_from(&mut fs::File::open(path).unwrap()).unwrap();
    assert_eq!(&reloaded, model);
    reloaded
}

fn likelihoods(result: &training::TrainingResult, expected: &str) {
    let rows: Vec<f32> = expected.lines().map(|line| line.parse().unwrap()).collect();
    assert_eq!(result.iterations.len(), rows.len());
    for (report, expected) in result.iterations.iter().zip(rows) {
        assert_eq!(report.file_likelihoods.len(), 1);
        assert_eq!(report.file_likelihoods[0].len(), 1);
        assert!((report.file_likelihoods[0][0] - expected).abs() <= 1e-5);
    }
}

#[test]
fn initialized_hmm_bootstrap_then_hsmm_matches_corrected_c_and_reloads() {
    let directory = Directory::new("hmm-bootstrap-hsmm");
    let definition: ModelDefinition =
        serde_json::from_str(include_str!("fixtures/init-definition.json")).unwrap();
    let empty = definition.build().unwrap();
    let initial: SegmentationDocument =
        serde_json::from_str(include_str!("fixtures/init-segmentation.json")).unwrap();
    let observation = dataset::read_observation(
        include_bytes!("fixtures/init-input.bin").as_slice(),
        &dataset::dimensions(&empty).unwrap(),
        12,
    )
    .unwrap();
    let corpus = Dataset {
        observations: vec![observation.clone()],
        segmentations: vec![dataset::read_segmentation(&initial.files[0].states, &empty).unwrap()],
    };
    let initialized =
        initialization::initialize(&empty, &corpus, initialization::Options::default()).unwrap();
    let initialized = saved_model(
        &initialized,
        &directory,
        "initialized.hsmm",
        include_bytes!("fixtures/init-c-aligned.hsmm"),
    );
    let document: SegmentationDocument =
        serde_json::from_str(include_str!("fixtures/align-c-isolated.json")).unwrap();
    let training_data = Dataset {
        observations: vec![observation.clone()],
        segmentations: vec![
            dataset::read_segmentation(&document.files[0].states, &initialized).unwrap(),
        ],
    };
    let mut options = training::Options {
        iterations: 2,
        termination_threshold: 0.0,
        duration_mode: DurationMode::Geometric,
        ..training::Options::default()
    };
    options.geometric.pruning_slope = 0.8;
    let bootstrap =
        training::train(&initialized, std::slice::from_ref(&training_data), options).unwrap();
    likelihoods(&bootstrap, include_str!("fixtures/rest-c-hmm.likelihood"));
    let bootstrap_model = saved_model(
        &bootstrap.model,
        &directory,
        "bootstrap.hsmm",
        include_bytes!("fixtures/rest-c-hmm.hsmm"),
    );
    options.duration_mode = DurationMode::Normal;
    let refined = training::train(&bootstrap_model, &[training_data], options).unwrap();
    likelihoods(
        &refined,
        include_str!("fixtures/rest-c-hmm-bootstrap-hsmm.likelihood"),
    );
    let reloaded = saved_model(
        &refined.model,
        &directory,
        "refined.hsmm",
        include_bytes!("fixtures/rest-c-hmm-bootstrap-hsmm.hsmm"),
    );
    for duration_mode in [
        alignment::DurationMode::Geometric,
        alignment::DurationMode::Explicit,
    ] {
        let settings = alignment::Options {
            duration_mode,
            ..alignment::Options::default()
        };
        let before = alignment::align_states(
            &refined.model,
            &observation,
            &document.files[0].states,
            settings,
        )
        .unwrap();
        let after =
            alignment::align_states(&reloaded, &observation, &document.files[0].states, settings)
                .unwrap();
        assert_eq!(
            serde_json::to_value(&before).unwrap(),
            serde_json::to_value(&after).unwrap()
        );
        assert_eq!(after.len(), document.files[0].states.len());
        assert_eq!(after.last().unwrap().time, 12.0);
    }
}
