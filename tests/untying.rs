mod common;
use common::Directory;
use liblrhsmm_rs::{EmissionPruning, Model};
use serde_json::json;
use shiro_rs::{dataset, labels::SegmentationDocument, untying};
use std::{
    fs,
    io::{self, Write},
    process::{Command, Stdio},
};

fn model() -> Model {
    Model::read_from(include_bytes!("fixtures/init-c-aligned.hsmm").as_slice()).unwrap()
}
fn document() -> SegmentationDocument {
    serde_json::from_str(include_str!("fixtures/align-c-isolated.json")).unwrap()
}

#[test]
fn original_c_model_json_and_summary_match() {
    let source = model();
    let document = document();
    let result = untying::untie(&source, &document).unwrap();
    let mut bytes = Vec::new();
    result.model.write_to(&mut bytes).unwrap();
    assert_eq!(bytes, include_bytes!("fixtures/untie-c.hsmm"));
    let expected: SegmentationDocument =
        serde_json::from_str(include_str!("fixtures/untie-c.json")).unwrap();
    assert_eq!(
        serde_json::to_value(&result.segmentation).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    bytes.clear();
    result.write_summary(&mut bytes).unwrap();
    assert_eq!(bytes, include_bytes!("fixtures/untie-c-summary.txt"));
    assert_eq!(
        Model::read_from(include_bytes!("fixtures/untie-c.hsmm").as_slice()).unwrap(),
        result.model
    );
    assert_eq!(source, model());
    assert_eq!(
        serde_json::to_value(document).unwrap(),
        serde_json::to_value(self::document()).unwrap()
    );
}

#[test]
fn weights_independent_copies_and_multifile_order_are_preserved() {
    let mut source = model();
    source.streams[0].weight = 0.25;
    source.streams[1].weight = 1.75;
    let mut encoded = Vec::new();
    source.write_to(&mut encoded).unwrap();
    assert_eq!(
        encoded,
        include_bytes!("fixtures/untie-c-weighted-input.hsmm")
    );
    let mut corrected = untying::untie(&source, &document()).unwrap().model;
    assert_eq!(corrected.streams[0].weight, 0.25);
    assert_eq!(corrected.streams[1].weight, 1.75);
    // The actual C tool loses both weights. Apart from this correction,
    // the resulting parameters must still reproduce its model exactly.
    corrected
        .streams
        .iter_mut()
        .for_each(|stream| stream.weight = 1.0);
    encoded.clear();
    corrected.write_to(&mut encoded).unwrap();
    assert_eq!(encoded, include_bytes!("fixtures/untie-c.hsmm"));
    let mut input = document();
    input.attributes.insert("corpus".into(), json!("retained"));
    input.files[0].states[0].jumps = Some(vec![json!({"d":2,"p":0.1})]);
    input.files[0].states[0]
        .attributes
        .insert("annotation".into(), json!("retained"));
    input.files.push(input.files[0].clone());
    let mut result = untying::untie(&source, &input).unwrap();
    assert_eq!(result.model.durations.len(), 12);
    assert_eq!(
        result.assignments[6],
        untying::Assignment {
            state: 6,
            file: 1,
            segment: 0
        }
    );
    assert_eq!(
        result.segmentation.files[1].states[0].outputs,
        Some(vec![6, 6])
    );
    assert_eq!(result.segmentation.attributes, input.attributes);
    assert_eq!(
        result.segmentation.files[0].states[0].jumps,
        input.files[0].states[0].jumps
    );
    assert_eq!(
        result.segmentation.files[0].states[0].attributes,
        input.files[0].states[0].attributes
    );
    let observation = dataset::read_observation(
        include_bytes!("fixtures/init-input.bin").as_slice(),
        &dataset::dimensions(&source).unwrap(),
        12,
    )
    .unwrap();
    let before = dataset::read_segmentation(&input.files[0].states, &source).unwrap();
    let after =
        dataset::read_segmentation(&result.segmentation.files[0].states, &result.model).unwrap();
    assert_eq!(
        source
            .emission_log_probabilities(&observation, &before, 1.0, EmissionPruning::None)
            .unwrap(),
        result
            .model
            .emission_log_probabilities(&observation, &after, 1.0, EmissionPruning::None)
            .unwrap()
    );
    assert_eq!(result.model.streams[0].weight, 0.25);
    assert_eq!(result.model.streams[1].weight, 1.75);
    result.model.streams[0].mixtures[0].means[0] = 100.0;
    result.model.durations[0].mean = 99.0;
    assert_eq!(
        result.model.streams[0].mixtures[6],
        source.streams[0].mixtures[0]
    );
    assert_eq!(result.model.durations[6], source.durations[0]);
    assert_ne!(
        result.model.streams[0].mixtures[0],
        source.streams[0].mixtures[0]
    );
}

#[test]
fn empty_metadata_validation_and_writer_errors_are_safe() {
    let source = model();
    let mut input = document();
    input.files.clear();
    let empty = untying::untie(&source, &input).unwrap();
    assert!(empty.model.durations.is_empty());
    assert!(
        empty
            .model
            .streams
            .iter()
            .all(|stream| stream.mixtures.is_empty())
    );
    let mut summary = Vec::new();
    empty.write_summary(&mut summary).unwrap();
    assert!(summary.is_empty());
    input = document();
    input.files[0].states[0].duration = Some(999);
    assert!(untying::untie(&source, &input).is_err());
    input = document();
    input.files[0].states[0].outputs = Some(vec![0]);
    assert!(untying::untie(&source, &input).is_err());
    input.files[0].states[0].outputs = Some(vec![0, 999]);
    assert!(untying::untie(&source, &input).is_err());
    input = document();
    input.files[0].states[0].metadata.clear();
    let result = untying::untie(&source, &input).unwrap();
    result.write_summary(&mut summary).unwrap();
    assert!(summary.starts_with(b"0 0 0\n"));
    input.files[0].states[0].metadata = vec![json!("a")];
    let result = untying::untie(&source, &input).unwrap();
    summary.clear();
    assert!(result.write_summary(&mut summary).is_err());
    assert!(summary.is_empty());
    struct FailedWriter;
    impl Write for FailedWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "test writer"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    assert_eq!(
        untying::untie(&source, &document())
            .unwrap()
            .write_summary(FailedWriter)
            .unwrap_err()
            .kind(),
        io::ErrorKind::BrokenPipe
    );
}

#[test]
fn cli_outputs_model_json_summary_and_accepts_model_stdin() {
    let directory = Directory::new("untie");
    let model_path = directory.path().join("model.hsmm");
    let input_path = directory.path().join("input.json");
    let json_path = directory.path().join("untied.json");
    let summary_path = directory.path().join("summary.txt");
    let model_bytes = include_bytes!("fixtures/init-c-aligned.hsmm");
    fs::write(&model_path, model_bytes).unwrap();
    fs::write(&input_path, serde_json::to_vec(&document()).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-untie"))
        .arg("-m")
        .arg(&model_path)
        .arg("-s")
        .arg(&input_path)
        .arg("-o")
        .arg(&json_path)
        .arg("-O")
        .arg(&summary_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, include_bytes!("fixtures/untie-c.hsmm"));
    assert_eq!(
        fs::read(summary_path).unwrap(),
        include_bytes!("fixtures/untie-c-summary.txt")
    );
    let actual: SegmentationDocument =
        serde_json::from_slice(&fs::read(json_path).unwrap()).unwrap();
    let expected: SegmentationDocument =
        serde_json::from_str(include_str!("fixtures/untie-c.json")).unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_shiro-untie"))
        .args(["-m", "-"])
        .arg("-s")
        .arg(&input_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(model_bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, include_bytes!("fixtures/untie-c.hsmm"));
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-untie"))
        .arg("-m")
        .arg(&model_path)
        .arg("-s")
        .arg(&input_path)
        .arg("-o")
        .arg(&input_path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    serde_json::from_slice::<SegmentationDocument>(&fs::read(input_path).unwrap()).unwrap();
    assert!(
        Command::new(env!("CARGO_BIN_EXE_shiro-untie"))
            .arg("-h")
            .output()
            .unwrap()
            .status
            .success()
    );
}
