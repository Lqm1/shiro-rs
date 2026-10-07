mod common;
use common::Directory;
use liblrhsmm_rs::Model;
use shiro_rs::{dataset, labels::SegmentationDocument};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

fn document() -> SegmentationDocument {
    serde_json::from_str(include_str!("fixtures/align-c-isolated.json")).unwrap()
}
fn run_model(flags: &[&str], expected: &[u8], likelihood: &str) {
    let directory = Directory::new("rest-modes");
    let model_path = directory.path().join("model.hsmm");
    let feature_path = directory.path().join("input.f");
    let input_path = directory.path().join("seg.json");
    let likelihood_path = directory.path().join("likelihood.csv");
    fs::write(&model_path, include_bytes!("fixtures/init-c-aligned.hsmm")).unwrap();
    fs::write(&feature_path, include_bytes!("fixtures/init-input.bin")).unwrap();
    let mut input = document();
    input.files[0].filename = feature_path.to_str().unwrap().to_owned();
    fs::write(&input_path, serde_json::to_vec(&input).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-rest"))
        .arg("-m")
        .arg(&model_path)
        .arg("-s")
        .arg(&input_path)
        .arg("-l")
        .arg(&likelihood_path)
        .args(flags)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{flags:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected, "{flags:?}");
    Model::read_from(output.stdout.as_slice()).unwrap();
    assert_eq!(
        fs::read_to_string(&likelihood_path).unwrap(),
        likelihood,
        "{flags:?}"
    );
    // The serialized model is usable for another expectation/update cycle.
    fs::write(&model_path, output.stdout).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-rest"))
        .arg("-m")
        .arg(&model_path)
        .arg("-s")
        .arg(&input_path)
        .args(flags)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Model::read_from(output.stdout.as_slice()).unwrap();
}

#[test]
fn original_flags_model_bytes_and_likelihood_csv_match_corrected_c() {
    run_model(
        &[],
        include_bytes!("fixtures/rest-c-hsmm-one.hsmm"),
        include_str!("fixtures/rest-c-hsmm-one.likelihood"),
    );
    run_model(
        &["-n", "2", "-t", "0"],
        include_bytes!("fixtures/rest-c-hsmm-two.hsmm"),
        include_str!("fixtures/rest-c-hsmm-two.likelihood"),
    );
    run_model(
        &["-n", "3", "-D", "-t", "0"],
        include_bytes!("fixtures/rest-c-daem.hsmm"),
        include_str!("fixtures/rest-c-daem.likelihood"),
    );
    run_model(
        &["-n", "2", "-g", "-P", "0.8", "-t", "0"],
        include_bytes!("fixtures/rest-c-hmm.hsmm"),
        include_str!("fixtures/rest-c-hmm.likelihood"),
    );
    run_model(
        &["-n", "2", "-i", "-t", "0"],
        include_bytes!("fixtures/rest-c-isolated.hsmm"),
        include_str!("fixtures/rest-c-isolated.likelihood"),
    );
    run_model(
        &["-M"],
        include_bytes!("fixtures/rest-c-hsmm-one.hsmm"),
        include_str!("fixtures/rest-c-mean.likelihood"),
    );
    run_model(
        &["-n", "2", "-i", "-M", "-t", "0"],
        include_bytes!("fixtures/rest-c-isolated-mean.hsmm"),
        include_str!("fixtures/rest-c-isolated-mean.likelihood"),
    );
    run_model(
        &["-n", "2", "-i", "-g", "-P", "0.8", "-t", "0"],
        include_bytes!("fixtures/rest-c-isolated-hmm.hsmm"),
        include_str!("fixtures/rest-c-isolated-hmm.likelihood"),
    );
    run_model(
        &["-n", "3", "-i", "-D", "-t", "0"],
        include_bytes!("fixtures/rest-c-isolated-daem.hsmm"),
        include_str!("fixtures/rest-c-isolated-daem.likelihood"),
    );
}

#[test]
fn stdin_parallel_flags_zero_iterations_and_output_aliases() {
    let directory = Directory::new("rest-input");
    let model_path = directory.path().join("model.hsmm");
    let input_path = directory.path().join("input.json");
    let feature_path = directory.path().join("input.f");
    let likelihood_path = directory.path().join("likelihood.csv");
    let model_bytes = include_bytes!("fixtures/init-c-aligned.hsmm");
    fs::write(&model_path, model_bytes).unwrap();
    fs::write(&feature_path, include_bytes!("fixtures/init-input.bin")).unwrap();
    let mut input = document();
    input.files[0].filename = feature_path.to_str().unwrap().to_owned();
    input.files.push(input.files[0].clone());
    fs::write(&input_path, serde_json::to_vec(&input).unwrap()).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_shiro-rest"))
        .args(["-m", "-"])
        .arg("-s")
        .arg(&input_path)
        .args(["-T", "-p", "2", "-d", "8", "-t", "-1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(model_bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Model::read_from(output.stdout.as_slice()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-rest"))
        .arg("-m")
        .arg(&model_path)
        .arg("-s")
        .arg(&input_path)
        .arg("-T")
        .arg("-l")
        .arg(&likelihood_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("multi-threading disabled"));
    assert_eq!(
        fs::read_to_string(&likelihood_path)
            .unwrap()
            .lines()
            .count(),
        2
    );
    for path in [&model_path, &input_path, &feature_path] {
        let before = fs::read(path).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_shiro-rest"))
            .arg("-m")
            .arg(&model_path)
            .arg("-s")
            .arg(&input_path)
            .arg("-l")
            .arg(path)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(fs::read(path).unwrap(), before);
    }
    fs::remove_file(feature_path).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-rest"))
        .arg("-m")
        .arg(&model_path)
        .arg("-s")
        .arg(&input_path)
        .args(["-n", "0"])
        .arg("-l")
        .arg(&likelihood_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, model_bytes);
    assert!(fs::read(likelihood_path).unwrap().is_empty());
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-rest"))
        .arg("-m")
        .arg(model_path)
        .arg("-s")
        .arg(input_path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        Command::new(env!("CARGO_BIN_EXE_shiro-rest"))
            .arg("-h")
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn isolated_json_loading_shares_alignment_boundaries_and_transition_fixes() {
    let model =
        Model::read_from(include_bytes!("fixtures/init-c-aligned.hsmm").as_slice()).unwrap();
    let observation = dataset::read_observation(
        include_bytes!("fixtures/init-input.bin").as_slice(),
        &dataset::dimensions(&model).unwrap(),
        12,
    )
    .unwrap();
    let mut states = document().files.remove(0).states;
    states[0].jumps = Some(vec![
        serde_json::json!({"d":4,"p":0.9}),
        serde_json::json!({"d":2,"p":0.1}),
    ]);
    let groups = dataset::isolated_groups(&model, &observation, &states).unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!((groups[1].first_state, groups[1].first_frame), (3, 6));
    assert_eq!(groups[0].observation.frames, 6);
    assert_eq!(groups[0].observation.frame(0, 0), observation.frame(0, 0));
    let segmentation = dataset::read_segmentation(&groups[0].states, &model).unwrap();
    assert_eq!(segmentation.boundaries, vec![2, 4, 6]);
    assert_eq!(segmentation.outgoing[0].len(), 2);
    assert_eq!(segmentation.outgoing[0][0].delta, 2);
    assert_eq!(segmentation.outgoing[0][0].probability, 0.1);
    states[5].time = 999.0;
    assert_eq!(
        dataset::isolated_groups(&model, &observation, &states).unwrap()[1]
            .observation
            .frames,
        6
    );
    states[0].metadata.clear();
    assert!(dataset::isolated_groups(&model, &observation, &states).is_err());
}
