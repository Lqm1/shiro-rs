mod common;
use ciglet_rs::resampling::BoundaryPolicy;
use common::Directory;
use shiro_rs::{
    audio::AudioOptions,
    batch::{self, Extractor, Options, Preset},
    index,
};
use std::fs;
#[test]
fn native_presets_match_original_lua_and_c() {
    let directory = Directory::new("presets");
    let stem = directory.path().join("clip with spaces.v1");
    fs::write(
        index::append_suffix(&stem, ".wav"),
        include_bytes!("fixtures/c-audio-input.wav"),
    )
    .unwrap();
    let cases = [
        (
            Preset::Mfcc12Da16k,
            include_bytes!("fixtures/c-fextr-mfcc12-da.bin").as_slice(),
            36,
        ),
        (
            Preset::Mfcc12Dae16k,
            include_bytes!("fixtures/c-fextr-mfcc12-dae.bin").as_slice(),
            39,
        ),
        (
            Preset::Plpcc12Da16k,
            include_bytes!("fixtures/c-fextr-plpcc12-da.bin").as_slice(),
            36,
        ),
    ];
    for (preset, expected, columns) in cases {
        let output = batch::extract_file(
            &stem,
            &Options::default(),
            &Extractor::Native(preset),
            || panic!("no dither expected"),
        )
        .unwrap();
        assert_eq!(
            fs::read(output.raw).unwrap(),
            include_bytes!("fixtures/c-audio-input.plain.raw")
        );
        assert!(output.mfcc.is_none());
        let actual = fs::read(output.parameters).unwrap();
        assert_eq!(actual.len(), 3 * columns * 4);
        assert_eq!(actual.len(), expected.len());
        let mut maximum = 0.0f64;
        for (actual, expected) in actual.chunks_exact(4).zip(expected.chunks_exact(4)) {
            let a = f32::from_le_bytes(actual.try_into().unwrap()) as f64;
            let e = f32::from_le_bytes(expected.try_into().unwrap()) as f64;
            let difference = (a - e).abs() / e.abs().max(1.0);
            maximum = maximum.max(difference);
            assert!(difference < 2e-5, "{preset:?}: {a} vs {e}");
        }
        eprintln!("Original fextr {preset:?}: maximum normalized difference {maximum:e}");
    }
}
#[test]
fn preparation_retains_stems_and_validates_before_output() {
    let directory = Directory::new("batch-preparation");
    let stem = directory.path().join("clip");
    fs::write(
        index::append_suffix(&stem, ".source.wav"),
        include_bytes!("fixtures/c-audio-input.wav"),
    )
    .unwrap();
    let options = Options {
        input_extension: ".source.wav".into(),
        audio: AudioOptions {
            normalize: true,
            output_sample_rate: Some(8000),
            boundary: BoundaryPolicy::LegacySkipFirst,
            ..Default::default()
        },
    };
    let output = batch::extract_file(
        &stem,
        &options,
        &Extractor::Native(Preset::default()),
        || panic!(),
    )
    .unwrap();
    assert_eq!(
        fs::read(output.raw).unwrap(),
        include_bytes!("fixtures/c-audio-input.normalized-down.raw")
    );
    assert_eq!(fs::read(output.parameters).unwrap().len(), 36 * 4);
    assert!(!index::append_suffix(&stem, ".source.raw").exists());
    let alias = directory.path().join("alias");
    let input = index::append_suffix(&alias, ".raw");
    fs::write(&input, include_bytes!("fixtures/c-audio-input.wav")).unwrap();
    assert!(
        batch::extract_file(
            &alias,
            &Options {
                input_extension: ".raw".into(),
                ..Default::default()
            },
            &Extractor::Native(Preset::default()),
            || panic!()
        )
        .is_err()
    );
    assert_eq!(
        fs::read(input).unwrap(),
        include_bytes!("fixtures/c-audio-input.wav")
    );
    assert!(
        batch::extract_file(
            &directory.path().join("missing"),
            &Options::default(),
            &Extractor::Native(Preset::default()),
            || panic!()
        )
        .is_err()
    );
    assert!(!directory.path().join("missing.raw").exists());
}
