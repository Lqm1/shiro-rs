mod common;
use common::Directory;
use serde_json::{Value, json};
use shiro_rs::labels::{self, PhoneMap, SegmentationDocument};
use std::{fs, process::Command};

fn phone_map() -> PhoneMap {
    serde_json::from_str(include_str!("fixtures/labels-phones.json")).unwrap()
}
fn original() -> SegmentationDocument {
    serde_json::from_str(include_str!("fixtures/labels-original-seg.json")).unwrap()
}
fn compare_rows(actual: &[labels::Label], expected: &str) {
    let expected = labels::parse(expected).unwrap();
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.name, expected.name);
        assert!((actual.start - expected.start).abs() < 1e-14);
        assert!((actual.end - expected.end).abs() < 1e-14);
    }
}
#[test]
fn conversion_matches_unchanged_lua_tools() {
    let input = labels::parse(include_str!("fixtures/labels-input.txt")).unwrap();
    let states = labels::to_states(&input, &phone_map(), 0.01).unwrap();
    let expected = original();
    assert_eq!(states.len(), 7);
    assert_eq!(states[2].time, 8.0); // Preserve source binary64 ceiling.
    for (actual, expected) in states.iter().zip(&expected.files[0].states) {
        assert_eq!(actual.time, expected.time);
        assert_eq!(actual.duration, expected.duration);
        assert_eq!(actual.outputs, expected.outputs);
        assert_eq!(actual.jumps, expected.jumps);
        assert_eq!(actual.metadata, expected.metadata);
    }
    compare_rows(
        &labels::from_states(&states, 0.01, false).unwrap(),
        include_str!("fixtures/labels-original-phones.txt"),
    );
    compare_rows(
        &labels::from_states(&states, 0.01, true).unwrap(),
        include_str!("fixtures/labels-original-states.txt"),
    );
    let mut output = Vec::new();
    labels::write(
        &labels::from_states(&states, 0.01, true).unwrap(),
        &mut output,
    )
    .unwrap();
    assert_eq!(output.iter().filter(|&&byte| byte == b'\r').count(), 10);
    compare_rows(
        &labels::parse(std::str::from_utf8(&output).unwrap()).unwrap(),
        include_str!("fixtures/labels-original-states.txt"),
    );
}
#[test]
fn parsing_validation_empty_states_and_metadata() {
    for (filename, expected) in [
        ("dir.name/clip.features.f", "dir.name/clip.features.txt"),
        ("dir.name\\clip", "dir.name\\clip.txt"),
        ("dir.name\\clip.f", "dir.name\\clip.txt"),
        (".f", ".txt"),
        ("clip", "clip.txt"),
    ] {
        assert_eq!(
            labels::output_path(filename, ".txt"),
            std::path::PathBuf::from(expected)
        );
    }
    assert_eq!(
        labels::parse("0  0.2 aa\r\n\r\n0.2\t0.3\tbb\textra\n")
            .unwrap()
            .len(),
        2
    );
    for text in [
        "0 1", "x 1 aa", "0 NaN aa", "0 inf aa", "-1 0 aa", "1 0 aa", "0\t1\t",
    ] {
        assert!(labels::parse(text).is_err(), "{text}");
    }
    assert!(
        labels::parse("0 1 aa\n\nx 2 bb")
            .unwrap_err()
            .to_string()
            .contains("line 3")
    );
    let input = labels::parse("0 0.1 empty").unwrap();
    assert!(
        labels::to_states(&input, &phone_map(), 0.01)
            .unwrap()
            .is_empty()
    );
    for hop in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(labels::to_states(&[], &phone_map(), hop).is_err());
        assert!(labels::from_states(&[], hop, false).is_err());
    }
    assert!(labels::to_states(&labels::parse("0 1 unknown").unwrap(), &phone_map(), 0.01).is_err());
    assert!(labels::to_states(&labels::parse("0 1e100 aa").unwrap(), &phone_map(), 0.01).is_err());
    let mut document = original();
    document
        .attributes
        .insert("source".into(), json!("fixture"));
    document.files[0].states[0]
        .metadata
        .push(json!({"extra":true}));
    let encoded = serde_json::to_value(&document).unwrap();
    let decoded: SegmentationDocument = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
    let mut states = original().files.remove(0).states;
    states[0].metadata.clear();
    assert!(labels::from_states(&states, 0.01, false).is_err());
    assert!(labels::from_states(&[], 0.01, true).unwrap().is_empty());
    states = original().files.remove(0).states;
    states[2].time = 0.0;
    assert!(labels::from_states(&states, 0.01, false).is_err());
}
#[test]
fn both_clis_preserve_options_paths_and_report_errors() {
    let directory = Directory::new("label-cli");
    fs::write(directory.path().join("index.csv"), "sample,aa aa bb\n").unwrap();
    fs::write(
        directory.path().join("phones.json"),
        include_bytes!("fixtures/labels-phones.json"),
    )
    .unwrap();
    fs::write(
        directory.path().join("sample.labels.txt"),
        include_bytes!("fixtures/labels-input.txt"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-lab2seg"))
        .current_dir(directory.path())
        .args([
            "index.csv",
            "-m",
            "phones.json",
            "-d",
            ".",
            "-e",
            ".labels.txt",
            "-E",
            ".features.f",
            "-t",
            "0.01",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: SegmentationDocument = serde_json::from_slice(&output.stdout).unwrap();
    assert!(document.files[0].filename.ends_with("sample.features.f"));
    assert_eq!(document.files[0].states[2].time, 8.0);
    fs::write(directory.path().join("seg.json"), &output.stdout).unwrap();
    for (option, extension, expected) in [
        (
            false,
            ".phones.txt",
            include_str!("fixtures/labels-original-phones.txt"),
        ),
        (
            true,
            ".states.txt",
            include_str!("fixtures/labels-original-states.txt"),
        ),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_shiro-seg2lab"));
        command
            .current_dir(directory.path())
            .args(["seg.json", "-t", "0.01", "-e", extension]);
        if option {
            command.arg("-s");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        compare_rows(
            &labels::parse(
                &fs::read_to_string(directory.path().join(format!("sample.features{extension}")))
                    .unwrap(),
            )
            .unwrap(),
            expected,
        );
    }
    fs::write(directory.path().join("sample.labels.txt"), "0 1 unknown").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-lab2seg"))
        .current_dir(directory.path())
        .args(["index.csv", "-m", "phones.json", "-e", ".labels.txt"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    fs::write(
        directory.path().join("bad.json"),
        serde_json::to_vec(
            &json!({"file_list":[{"filename":"untouched.f", "states":[{"time":1,"ext":[]}]}]}),
        )
        .unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-seg2lab"))
        .current_dir(directory.path())
        .arg("bad.json")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!directory.path().join("untouched.txt").exists());
    let alias = json!({"file_list":[{"filename":"./alias.f", "states":[]}]});
    let alias_bytes = serde_json::to_vec(&alias).unwrap();
    fs::write(directory.path().join("alias.json"), &alias_bytes).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-seg2lab"))
        .current_dir(directory.path())
        .args(["alias.json", "-e", ".json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        fs::read(directory.path().join("alias.json")).unwrap(),
        alias_bytes
    );
    for program in [
        env!("CARGO_BIN_EXE_shiro-lab2seg"),
        env!("CARGO_BIN_EXE_shiro-seg2lab"),
    ] {
        assert!(
            Command::new(program)
                .arg("--help")
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let _: Value =
        serde_json::from_slice(&fs::read(directory.path().join("seg.json")).unwrap()).unwrap();
}
