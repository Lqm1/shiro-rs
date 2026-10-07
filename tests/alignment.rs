mod common;
use common::Directory;
use liblrhsmm_rs::Model;
use serde_json::json;
use shiro_rs::{
    alignment::{self, DurationMode, Options},
    dataset,
    labels::SegmentationDocument,
};
use std::{fs, process::Command};

fn model() -> Model {
    Model::read_from(include_bytes!("fixtures/init-c-aligned.hsmm").as_slice()).unwrap()
}
fn observation(model: &Model) -> liblrhsmm_rs::Observation {
    dataset::read_observation(
        include_bytes!("fixtures/init-input.bin").as_slice(),
        &dataset::dimensions(model).unwrap(),
        12,
    )
    .unwrap()
}
fn source(isolated: bool) -> SegmentationDocument {
    serde_json::from_str(if isolated {
        include_str!("fixtures/align-c-isolated.json")
    } else {
        include_str!("fixtures/align-c-embedded.json")
    })
    .unwrap()
}
#[test]
fn eight_modes_match_original_c_state_paths() {
    let model = model();
    let observation = observation(&model);
    let references = [
        include_str!("fixtures/align-c-embedded-hsmm.json"),
        include_str!("fixtures/align-c-embedded-hmm.json"),
        include_str!("fixtures/align-c-embedded-hsmm-pruned.json"),
        include_str!("fixtures/align-c-embedded-hmm-pruned.json"),
        include_str!("fixtures/align-c-isolated-hsmm.json"),
        include_str!("fixtures/align-c-isolated-hmm.json"),
        include_str!("fixtures/align-c-isolated-hsmm-pruned.json"),
        include_str!("fixtures/align-c-isolated-hmm-pruned.json"),
    ];
    for (index, reference) in references.iter().enumerate() {
        let isolated = index >= 4;
        let mut options = Options {
            isolated,
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
        let input = source(isolated);
        let expected: SegmentationDocument = serde_json::from_str(reference).unwrap();
        let actual =
            alignment::align_states(&model, &observation, &input.files[0].states, options).unwrap();
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(&expected.files[0].states).unwrap(),
            "case {index}"
        );
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(source(isolated)).unwrap()
        );
    }
}

#[test]
fn default_hmm_pruning_matches_c_when_a_path_exists() {
    let model = model();
    let observation = observation(&model);
    let input: SegmentationDocument =
        serde_json::from_str(include_str!("fixtures/align-c-four.json")).unwrap();
    let expected: SegmentationDocument =
        serde_json::from_str(include_str!("fixtures/align-c-four-hmm.json")).unwrap();
    let options = Options {
        duration_mode: DurationMode::Geometric,
        ..Options::default()
    };
    let actual =
        alignment::align_states(&model, &observation, &input.files[0].states, options).unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(&expected.files[0].states).unwrap()
    );
    assert!(
        alignment::align_states(
            &model,
            &observation,
            &source(false).files[0].states,
            options
        )
        .is_err()
    );
}

#[test]
fn isolated_boundaries_and_filtered_jumps_are_safe() {
    let model = model();
    let observation = observation(&model);
    let options = Options {
        isolated: true,
        ..Options::default()
    };
    let mut document = source(true);
    let expected =
        alignment::align_states(&model, &observation, &document.files[0].states, options).unwrap();
    // The first extra edge is invalid; later valid edges must not leave a hole.
    // A zero-probability local edge makes filtering independent of inference scores.
    document.files[0].states[2].jumps = Some(vec![json!({"d":2,"p":0.9}), json!({"d":0,"p":0.0})]);
    let actual =
        alignment::align_states(&model, &observation, &document.files[0].states, options).unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    document.files[0].states[2].jumps = None;
    document.files[0].states[5].time = 999.0;
    let aligned =
        alignment::align_states(&model, &observation, &document.files[0].states, options).unwrap();
    assert_eq!(aligned.last().unwrap().time, 12.0);
    document.files[0].states[5].time = 5.0;
    assert!(
        alignment::align_states(&model, &observation, &document.files[0].states, options).is_err()
    );
    document = source(true);
    document.files[0].states[0].metadata.clear();
    assert!(
        alignment::align_states(&model, &observation, &document.files[0].states, options).is_err()
    );
    // Each one-state phoneme must retain its own original interval.
    let mut states = source(false).files.remove(0).states;
    for state in &mut states {
        state.metadata = vec![json!("repeated"), json!(0)];
        state.jumps = None;
    }
    let aligned = alignment::align_states(&model, &observation, &states, options).unwrap();
    assert_eq!(
        aligned.iter().map(|s| s.time).collect::<Vec<_>>(),
        vec![3.0, 7.0, 12.0]
    );
}

#[test]
fn alignment_cli_roundtrips_json_and_reports_invalid_inputs() {
    let directory = Directory::new("alignment");
    let model_path = directory.path().join("model.hsmm");
    let feature_path = directory.path().join("input.f");
    let segmentation_path = directory.path().join("seg.json");
    fs::write(&model_path, include_bytes!("fixtures/init-c-aligned.hsmm")).unwrap();
    fs::write(&feature_path, include_bytes!("fixtures/init-input.bin")).unwrap();
    for flags in [
        vec![],
        vec!["-g", "-P", "0.8"],
        vec!["-i"],
        vec!["-i", "-g", "-P", "0.8"],
        vec!["-p", "2", "-d", "8"],
    ] {
        let mut document = source(flags.contains(&"-i"));
        document.files[0].filename = feature_path.to_str().unwrap().to_owned();
        document
            .attributes
            .insert("corpus".into(), json!("retained"));
        fs::write(&segmentation_path, serde_json::to_vec(&document).unwrap()).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_shiro-align"))
            .args(["-m"])
            .arg(&model_path)
            .arg("-s")
            .arg(&segmentation_path)
            .args(&flags)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: SegmentationDocument = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result.attributes, document.attributes);
        assert_eq!(result.files[0].filename, document.files[0].filename);
        assert_eq!(result.files[0].states.last().unwrap().time, 12.0);
        dataset::read_segmentation(&result.files[0].states, &model()).unwrap();
    }
    fs::write(feature_path, [0, 1, 2]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-align"))
        .arg("-m")
        .arg(model_path)
        .arg("-s")
        .arg(segmentation_path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        Command::new(env!("CARGO_BIN_EXE_shiro-align"))
            .arg("-h")
            .output()
            .unwrap()
            .status
            .success()
    );
}
