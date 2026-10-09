mod common;
use common::Directory;
use liblrhsmm_rs::{Dataset, Model};
use serde_json::json;
use shiro_rs::{
    dataset,
    definition::ModelDefinition,
    initialization::{self, Options},
    labels::SegmentationDocument,
};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};
fn model() -> Model {
    serde_json::from_str::<ModelDefinition>(include_str!(
        "../../../tests/fixtures/init-definition.json"
    ))
    .unwrap()
    .build()
    .unwrap()
}
fn document() -> SegmentationDocument {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/init-segmentation.json"
    ))
    .unwrap()
}
fn fixture_data() -> Dataset {
    let model = model();
    Dataset {
        observations: vec![
            dataset::read_observation(
                include_bytes!("../../../tests/fixtures/init-input.bin").as_slice(),
                &dataset::dimensions(&model).unwrap(),
                12,
            )
            .unwrap(),
        ],
        segmentations: vec![
            dataset::read_segmentation(&document().files[0].states, &model).unwrap(),
        ],
    }
}
#[test]
fn four_initialization_modes_match_original_c_model_bytes() {
    let source = model();
    let data = fixture_data();
    let cases = [
        (
            Options::default(),
            include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm").as_slice(),
        ),
        (
            Options {
                flat_start: true,
                ..Options::default()
            },
            include_bytes!("../../../tests/fixtures/init-c-flat.hsmm").as_slice(),
        ),
        (
            Options {
                globally_tied: true,
                ..Options::default()
            },
            include_bytes!("../../../tests/fixtures/init-c-tied.hsmm").as_slice(),
        ),
        (
            Options {
                flat_start: true,
                globally_tied: true,
                variance_floor_ratio: 0.4,
            },
            include_bytes!("../../../tests/fixtures/init-c-flat-tied.hsmm").as_slice(),
        ),
    ];
    for (options, expected) in cases {
        let initialized = initialization::initialize(&source, &data, options).unwrap();
        let mut encoded = Vec::new();
        initialized.write_to(&mut encoded).unwrap();
        assert_eq!(encoded, expected, "{options:?}");
        assert_eq!(Model::read_from(encoded.as_slice()).unwrap(), initialized);
    }
    assert_eq!(source, model());
    assert_eq!(data, fixture_data());
    let mut heterogeneous = source.clone();
    heterogeneous.streams[0].mixtures[2] = liblrhsmm_rs::GaussianMixture::new(1, 5).unwrap();
    assert_eq!(dataset::dimensions(&heterogeneous).unwrap(), vec![2, 1]);
    let tied = initialization::initialize(
        &heterogeneous,
        &data,
        Options {
            globally_tied: true,
            ..Options::default()
        },
    )
    .unwrap();
    let mut encoded = Vec::new();
    tied.write_to(&mut encoded).unwrap();
    assert_eq!(
        encoded,
        include_bytes!("../../../tests/fixtures/init-c-tied.hsmm")
    );
    let mut ten = data.clone();
    ten.observations[0] = dataset::read_observation(
        &include_bytes!("../../../tests/fixtures/init-input.bin")[..120],
        &[2, 1],
        10,
    )
    .unwrap();
    let initialized = initialization::initialize(
        &source,
        &ten,
        Options {
            flat_start: true,
            ..Options::default()
        },
    )
    .unwrap();
    let mut encoded = Vec::new();
    initialized.write_to(&mut encoded).unwrap();
    // C flat-start FP_TYPE duration 10/3 ends at frame 9, retaining its rounding.
    assert_eq!(
        encoded,
        include_bytes!("../../../tests/fixtures/init-c-flat-ten.hsmm")
    );
}
#[test]
fn complete_frames_optional_metadata_and_transition_precision() {
    let source = model();
    let data = fixture_data();
    assert_eq!(
        data.observations[0].streams[0].values[0..4],
        [-5.0, -1.5, -4.0, -0.5]
    );
    assert_eq!(
        data.observations[0].streams[1].values[0..3],
        [0.0, 0.25, 0.5]
    );
    // Original cli-common.h driver, FP_TYPE=float: 0.02, 0.15, residual.
    assert_eq!(
        data.segmentations[0].outgoing[0][2].probability.to_bits(),
        0x3f54_7ae2
    );
    let missing: SegmentationDocument = serde_json::from_value(
        json!({"file_list":[{"filename":"unused", "states":[{"dur":0,"out":[0,0]}]}]}),
    )
    .unwrap();
    assert_eq!(
        dataset::read_segmentation(&missing.files[0].states, &source)
            .unwrap()
            .boundaries,
        vec![0]
    );
    let mut states = document().files.remove(0).states;
    states[0].time = 3.9;
    states[0]
        .jumps
        .as_mut()
        .unwrap()
        .push(json!({"d":1,"p":0.9}));
    let segmentation = dataset::read_segmentation(&states, &source).unwrap();
    assert_eq!(segmentation.boundaries[0], 3);
    assert_eq!(segmentation.outgoing[0].len(), 3);
    states[0].jumps = Some(vec![json!({"d":0,"p":0.1}), json!({"d":-1,"p":0.2})]);
    let signed = dataset::read_segmentation(&states, &source).unwrap();
    assert_eq!(
        signed.outgoing[0]
            .iter()
            .map(|jump| jump.delta)
            .collect::<Vec<_>>(),
        vec![0, -1, 1]
    );
    assert_eq!(
        signed.outgoing[0]
            .iter()
            .map(|jump| jump.probability.to_bits())
            .collect::<Vec<_>>(),
        vec![0x3dcc_cccd, 0x3e4c_cccd, 0x3f33_3333]
    );
    assert_eq!(
        dataset::read_observation(
            include_bytes!("../../../tests/fixtures/init-input.bin").as_slice(),
            &[2, 1],
            i32::MAX as usize
        )
        .unwrap()
        .frames,
        12
    );
    for bytes in [&[0u8; 3][..], &[0u8; 8][..]] {
        assert!(dataset::read_observation(bytes, &[2, 1], 12).is_err());
    }
    assert!(
        dataset::read_observation(
            include_bytes!("../../../tests/fixtures/init-input.bin").as_slice(),
            &[2, 1],
            11
        )
        .is_err()
    );
    let dimensions = vec![1usize; 70];
    let observation =
        dataset::read_observation(vec![0u8; 70 * 4].as_slice(), &dimensions, 1).unwrap();
    assert_eq!(observation.streams.len(), 70);
    states[0].jumps = Some(vec![json!({"d":2,"p":0.8}), json!({"d":3,"p":0.8})]);
    assert!(dataset::read_segmentation(&states, &source).is_err());
}
#[test]
fn corpus_fallback_counts_all_files_and_clamps_supplied_boundaries() {
    let mut data = fixture_data();
    data.observations.push(data.observations[0].clone());
    data.segmentations.push(data.segmentations[0].clone());
    data.segmentations[1].boundaries[2] = 999;
    let initialized = initialization::initialize(&model(), &data, Options::default()).unwrap();
    assert_eq!(initialized.durations[3].mean, 4.0);
    assert_eq!(initialized.durations[3].variance, 16.0);
    let mut expected =
        Model::read_from(include_bytes!("../../../tests/fixtures/init-c-multi.hsmm").as_slice())
            .unwrap();
    assert_eq!(expected.durations[3].mean, 8.0);
    assert_eq!(expected.durations[3].variance, 64.0);
    expected.durations[3].mean = 4.0;
    expected.durations[3].variance = 16.0;
    assert_eq!(initialized, expected);
    assert!(initialization::initialize(&model(), &Dataset::default(), Options::default()).is_err());
    assert!(
        initialization::initialize(
            &model(),
            &data,
            Options {
                variance_floor_ratio: f32::NAN,
                ..Options::default()
            }
        )
        .is_err()
    );
    data.segmentations[0].boundaries[1] = -1;
    assert!(initialization::initialize(&model(), &data, Options::default()).is_err());
}
#[test]
fn cli_initializes_binary_models_and_rejects_partial_scalars() {
    let directory = Directory::new("init-cli");
    let source = model();
    source
        .write_to(fs::File::create(directory.path().join("empty.hsmm")).unwrap())
        .unwrap();
    fs::write(
        directory.path().join("input.f"),
        include_bytes!("../../../tests/fixtures/init-input.bin"),
    )
    .unwrap();
    fs::write(
        directory.path().join("seg.json"),
        include_bytes!("../../../tests/fixtures/init-segmentation.json"),
    )
    .unwrap();
    for (flags, expected) in [
        (
            vec![],
            include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm").as_slice(),
        ),
        (
            vec!["-F"],
            include_bytes!("../../../tests/fixtures/init-c-flat.hsmm").as_slice(),
        ),
        (
            vec!["-T"],
            include_bytes!("../../../tests/fixtures/init-c-tied.hsmm").as_slice(),
        ),
        (
            vec!["-FT", "-v", "0.4"],
            include_bytes!("../../../tests/fixtures/init-c-flat-tied.hsmm").as_slice(),
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_shiro-init"))
            .current_dir(directory.path())
            .args(["-m", "empty.hsmm", "-s", "seg.json"])
            .args(flags)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected);
    }
    let mut pipe = Command::new(env!("CARGO_BIN_EXE_shiro-init"))
        .current_dir(directory.path())
        .args(["-m", "-", "-s", "seg.json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut bytes = Vec::new();
    source.write_to(&mut bytes).unwrap();
    {
        let mut stdin = pipe.stdin.take().unwrap();
        stdin.write_all(&bytes).unwrap();
    }
    let output = pipe.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm")
    );
    fs::write(directory.path().join("input.f"), [0u8; 3]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-init"))
        .current_dir(directory.path())
        .args(["-m", "empty.hsmm", "-s", "seg.json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        Command::new(env!("CARGO_BIN_EXE_shiro-init"))
            .arg("--help")
            .output()
            .unwrap()
            .status
            .success()
    );
}
