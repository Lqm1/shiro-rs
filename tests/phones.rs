mod common;
use common::Directory;
use serde::Deserialize;
use serde_json::{Value, json};
use shiro_rs::{
    definition::ModelDefinition,
    labels::{PhoneMap, SegmentationDocument},
    phonemap::{self, Options},
    segmentation,
};
use std::{fs, process::Command};

#[test]
fn feature_frame_sizes_keep_native_dimension_and_platform_limits() {
    assert_eq!(segmentation::feature_frame_count(0, 36).unwrap(), 0);
    assert_eq!(segmentation::feature_frame_count(144 * 12, 36).unwrap(), 12);
    for (bytes, dimensions) in [
        (1, 36),
        (143, 36),
        (145, 36),
        (0, 0),
        (0, i32::MAX as usize + 1),
    ] {
        assert!(segmentation::feature_frame_count(bytes, dimensions).is_err());
    }
    assert_eq!(
        segmentation::feature_frame_count(u32::MAX as u64 * 4, 1).unwrap(),
        u32::MAX as usize
    );
    if usize::BITS == 32 {
        assert!(segmentation::feature_frame_count((u32::MAX as u64 + 1) * 4, 1).is_err());
    } else {
        assert_eq!(
            segmentation::feature_frame_count((u32::MAX as u64 + 1) * 4, 1).unwrap() as u64,
            u32::MAX as u64 + 1
        );
    }
}
#[derive(Deserialize)]
struct Oracle {
    topology: String,
    count: usize,
    map: PhoneMap,
    definition: Option<ModelDefinition>,
    segmentation: SegmentationDocument,
}
fn cases() -> Vec<Oracle> {
    serde_json::from_str(include_str!("fixtures/phones-original.json")).unwrap()
}
fn assert_json(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(e)) => {
            assert!((a.as_f64().unwrap() - e.as_f64().unwrap()).abs() <= 1e-14)
        }
        (Value::Array(a), Value::Array(e)) => {
            assert_eq!(a.len(), e.len());
            for (a, e) in a.iter().zip(e) {
                assert_json(a, e);
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            assert_eq!(a.len(), e.len());
            for (key, a) in a {
                assert_json(a, &e[key]);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}
#[test]
fn all_original_topologies_and_short_phones_match_lua() {
    for case in cases() {
        let options = Options {
            states_per_phone: case.count,
            streams: 2,
            topology: Some(case.topology.clone()),
            weak_skips: true,
        };
        let map = phonemap::create(include_str!("fixtures/phones-input.txt"), &options).unwrap();
        assert_json(
            &serde_json::to_value(&map).unwrap(),
            &serde_json::to_value(&case.map).unwrap(),
        );
        let names: Vec<_> = ["bb", "aa", "bb", "cc", "aa"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let states = segmentation::initial(&names, &map, 41).unwrap();
        for (a, e) in states.iter().zip(&case.segmentation.files[0].states) {
            assert_eq!(a.time, e.time);
            assert_eq!(a.duration, e.duration);
            assert_eq!(a.outputs, e.outputs);
            assert_eq!(a.metadata, e.metadata);
            assert_json(
                &serde_json::to_value(&a.jumps).unwrap(),
                &serde_json::to_value(&e.jumps).unwrap(),
            );
        }
        assert_eq!(states.len(), case.segmentation.files[0].states.len());
        if let Some(mut expected) = case.definition {
            let actual = phonemap::to_definition(&map, 12, 0.01).unwrap();
            expected
                .duration_constraints
                .sort_by_key(|constraint| constraint.index);
            assert_json(
                &serde_json::to_value(&actual).unwrap(),
                &serde_json::to_value(&expected).unwrap(),
            );
            assert_eq!(actual.build().unwrap().streams.len(), 2);
        }
    }
}
#[test]
fn defaults_errors_shared_constraints_and_immutable_maps() {
    let default = phonemap::create("aa\nbb\n", &Options::default()).unwrap();
    assert_eq!(default.phones["bb"].states[0].duration, 3);
    assert_eq!(default.phones["aa"].states[0].outputs, vec![0; 3]);
    assert!(default.phones["aa"].attributes.is_empty());
    assert_eq!(
        phonemap::create("aa\n\nbb\r\n", &Options::default())
            .unwrap()
            .phones
            .len(),
        2
    );
    for text in ["aa\naa", "aa durfloor", "aa durceil NaN", "aa durfloor inf"] {
        assert!(phonemap::create(text, &Options::default()).is_err());
    }
    let zero = Options {
        states_per_phone: 0,
        ..Options::default()
    };
    assert!(phonemap::create("aa", &zero).is_err());
    let mut map: PhoneMap = serde_json::from_value(json!({"phone_map":{
        "aa":{"states":[{"dur":2,"out":[4,0]}],"durfloor":[0.02],"durceil":[0.1]},
        "bb":{"states":[{"dur":2,"out":[1,6]}],"durfloor":[0.03],"durceil":[0.08]}
    }}))
    .unwrap();
    let before = serde_json::to_value(&map).unwrap();
    let definition = phonemap::to_definition(&map, 12, 0.01).unwrap();
    assert_eq!(definition.duration_states, 3);
    assert_eq!(
        definition
            .streams
            .iter()
            .map(|stream| stream.states)
            .collect::<Vec<_>>(),
        vec![5, 7]
    );
    assert_eq!(definition.duration_constraints[0].minimum, Some(3));
    assert_eq!(definition.duration_constraints[0].maximum, Some(8));
    assert_eq!(serde_json::to_value(&map).unwrap(), before);
    map.phones
        .get_mut("bb")
        .unwrap()
        .attributes
        .insert("durceil".into(), json!([-1]));
    assert_eq!(
        phonemap::to_definition(&map, 12, 0.01)
            .unwrap()
            .duration_constraints[0]
            .maximum,
        Some(10)
    );
    map.phones
        .get_mut("bb")
        .unwrap()
        .attributes
        .insert("durceil".into(), json!([0.08]));
    map.phones
        .get_mut("bb")
        .unwrap()
        .attributes
        .insert("durfloor".into(), json!([0.2]));
    assert!(phonemap::to_definition(&map, 12, 0.01).is_err());
    assert!(phonemap::to_definition(&map, 0, 0.01).is_err());
    assert!(phonemap::to_definition(&map, 12, 0.0).is_err());
    assert!(segmentation::initial(&[], &map, 10).unwrap().is_empty());
    assert!(segmentation::initial(&["unknown".into()], &map, 10).is_err());
    let mut map = default;
    map.phones
        .get_mut("aa")
        .unwrap()
        .attributes
        .insert("pskip".into(), json!(1.1));
    assert!(segmentation::initial(&["aa".into()], &map, 10).is_err());
}
#[test]
fn three_cli_workflows_create_loadable_models_and_segmentations() {
    let directory = Directory::new("phone-cli");
    fs::write(
        directory.path().join("phones.txt"),
        include_bytes!("fixtures/phones-input.txt"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-mkpm"))
        .current_dir(directory.path())
        .args(["phones.txt", "-s", "4", "-S", "2", "-t", "type-c", "-w"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let map: PhoneMap = serde_json::from_slice(&output.stdout).unwrap();
    fs::write(directory.path().join("map.json"), &output.stdout).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-pm2md"))
        .current_dir(directory.path())
        .args(["map.json", "-d", "12", "-t", "0.01"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let definition: ModelDefinition = serde_json::from_slice(&output.stdout).unwrap();
    fs::write(directory.path().join("definition.json"), &output.stdout).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-mkhsmm"))
        .current_dir(directory.path())
        .args(["-c", "definition.json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let model = liblrhsmm_rs::Model::read_from(output.stdout.as_slice()).unwrap();
    assert_eq!(model.durations.len(), 12);
    assert_eq!(model.streams.len(), 2);
    assert_eq!(model, definition.build().unwrap());
    fs::write(directory.path().join("index.csv"), "clip,aa bb cc\n").unwrap();
    fs::write(
        directory.path().join("clip.features.f"),
        vec![0u8; 41 * 36 * 4],
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-mkseg"))
        .current_dir(directory.path())
        .args([
            "index.csv",
            "-m",
            "map.json",
            "-n",
            "36",
            "-e",
            ".features.f",
            "-L",
            "bb",
            "-R",
            "aa",
            "-t",
            "0.05",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: SegmentationDocument = serde_json::from_slice(&output.stdout).unwrap();
    let names: Vec<_> = ["bb", "aa", "bb", "cc", "aa"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_json(
        &serde_json::to_value(&document.files[0].states).unwrap(),
        &serde_json::to_value(segmentation::initial(&names, &map, 41).unwrap()).unwrap(),
    );
    fs::write(directory.path().join("clip.features.f"), [0; 3]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-mkseg"))
        .current_dir(directory.path())
        .args(["index.csv", "-m", "map.json", "-e", ".features.f"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    for program in [
        env!("CARGO_BIN_EXE_shiro-mkpm"),
        env!("CARGO_BIN_EXE_shiro-pm2md"),
        env!("CARGO_BIN_EXE_shiro-mkseg"),
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
}
