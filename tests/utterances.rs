use liblrhsmm_rs::Model;
use shiro_rs::{
    features::Features,
    labels::SegmentationDocument,
    utterances::{self, ModelSource, Options},
};

fn features() -> Features {
    Features {
        frames: 40,
        columns: 13,
        values: include_bytes!("fixtures/utterances-c-features.bin")
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
            .collect(),
    }
}
fn options() -> Options {
    Options {
        utterances: 2,
        iterations: 2,
        ..Options::default()
    }
}
fn encode(model: &Model) -> Vec<u8> {
    let mut bytes = Vec::new();
    model.write_to(&mut bytes).unwrap();
    bytes
}

#[test]
fn waveform_to_features_training_and_alignment_matches_reference() {
    let wave = ciglet_rs::wave::read(
        &mut std::io::Cursor::new(include_bytes!("fixtures/utterances-input.wav")),
        i32::MAX as usize,
    )
    .unwrap();
    let mut random = shiro_rs::audio::DitherSequence::linux_gnu();
    let result = utterances::split_wave(
        wave,
        "sample.param",
        13,
        shiro_rs::features::FeatureKind::Mfcc,
        options(),
        ModelSource::Fresh,
        || random.next_uniform(),
    )
    .unwrap();
    let expected_audio: Vec<_> = include_bytes!("fixtures/utterances-c-audio.bin")
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();
    assert_eq!(result.audio.sample_rate, 16000);
    assert_eq!(result.audio.samples, expected_audio);
    let reference = features();
    assert_eq!(result.features.frames, reference.frames);
    assert_eq!(result.features.columns, reference.columns);
    let maximum = result
        .features
        .values
        .iter()
        .zip(&reference.values)
        .map(|(actual, expected)| (actual - expected).abs() / expected.abs().max(1.0))
        .fold(0.0f32, f32::max);
    eprintln!("wavsplit feature maximum normalized difference: {maximum}");
    assert!(maximum <= 2e-5);
    let expected =
        utterances::split_features(&reference, "sample.param", options(), ModelSource::Fresh)
            .unwrap();
    assert_eq!(result.utterances.labels, expected.labels);
    let mut bytes = Vec::new();
    result.utterances.model.write_to(&mut bytes).unwrap();
    let reloaded = Model::read_from(bytes.as_slice()).unwrap();
    let aligned = utterances::split_features(
        &result.features,
        "sample.param",
        options(),
        ModelSource::Trained(&reloaded),
    )
    .unwrap();
    assert_eq!(aligned.labels, result.utterances.labels);
}

#[test]
fn fresh_intermediate_models_and_alignment_match_corrected_c_lua() {
    let result =
        utterances::split_features(&features(), "sample.param", options(), ModelSource::Fresh)
            .unwrap();
    assert_eq!(
        encode(result.uninitialized_model.as_ref().unwrap()),
        include_bytes!("fixtures/utterances-c-uninit.hsmm")
    );
    assert_eq!(
        encode(result.initialized_model.as_ref().unwrap()),
        include_bytes!("fixtures/utterances-c-flat.hsmm")
    );
    assert_eq!(
        encode(&result.model),
        include_bytes!("fixtures/utterances-c-trained.hsmm")
    );
    assert_eq!(result.iterations.len(), 2);
    assert_eq!(result.phones, ["sil", "utt", "sil", "utt", "sil"]);
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/utterances-c-phonemap.json")).unwrap();
    assert_eq!(serde_json::to_value(result.phonemap).unwrap(), expected);
    let definition: shiro_rs::definition::ModelDefinition =
        serde_json::from_str(include_str!("fixtures/utterances-c-definition.json")).unwrap();
    assert_eq!(
        result.definition.build().unwrap(),
        definition.build().unwrap()
    );
    for (actual, reference) in [
        (
            &result.initial_segmentation,
            include_str!("fixtures/utterances-c-initial.json"),
        ),
        (
            &result.alignment,
            include_str!("fixtures/utterances-c-aligned.json"),
        ),
    ] {
        let expected: SegmentationDocument = serde_json::from_str(reference).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }
    for (actual, row) in result
        .labels
        .iter()
        .zip(include_str!("fixtures/utterances-c-labels.txt").lines())
    {
        let expected: Vec<_> = row.split('\t').collect();
        assert!((actual.start - expected[0].parse::<f64>().unwrap()).abs() < 1e-14);
        assert!((actual.end - expected[1].parse::<f64>().unwrap()).abs() < 1e-14);
        assert_eq!(actual.name, expected[2]);
    }
    assert_eq!(result.labels.len(), 5);
}

#[test]
fn initialized_and_trained_sources_preserve_input_models() {
    let initialized =
        Model::read_from(include_bytes!("fixtures/utterances-c-flat.hsmm").as_slice()).unwrap();
    let trained =
        Model::read_from(include_bytes!("fixtures/utterances-c-trained.hsmm").as_slice()).unwrap();
    let initialized_bytes = encode(&initialized);
    let trained_bytes = encode(&trained);
    let result = utterances::split_features(
        &features(),
        "sample.param",
        options(),
        ModelSource::Initialized(&initialized),
    )
    .unwrap();
    assert_eq!(encode(&result.model), trained_bytes);
    assert_eq!(result.iterations.len(), 2);
    assert!(result.uninitialized_model.is_none());
    assert!(result.initialized_model.is_none());
    let loaded = utterances::split_features(
        &features(),
        "sample.param",
        options(),
        ModelSource::Trained(&trained),
    )
    .unwrap();
    assert_eq!(loaded.labels, result.labels);
    assert!(loaded.iterations.is_empty());
    assert!(loaded.uninitialized_model.is_none());
    assert!(loaded.initialized_model.is_none());
    assert_eq!(encode(&initialized), initialized_bytes);
    assert_eq!(encode(&trained), trained_bytes);
}

#[test]
fn invalid_counts_feature_shapes_and_models_return_errors() {
    for settings in [
        Options {
            utterances: usize::MAX,
            ..options()
        },
        Options {
            hop_seconds: 0.0,
            ..options()
        },
        Options {
            hop_seconds: 1e-30,
            ..options()
        },
        Options {
            minimum_silence_seconds: f64::NAN,
            ..options()
        },
        Options {
            iterations: usize::MAX,
            ..options()
        },
    ] {
        assert!(
            utterances::split_features(&features(), "sample.param", settings, ModelSource::Fresh)
                .is_err()
        );
    }
    let mut invalid = features();
    invalid.values.pop();
    assert!(
        utterances::split_features(&invalid, "sample.param", options(), ModelSource::Fresh)
            .is_err()
    );
    let mut invalid = features();
    invalid.values[0] = f32::NAN;
    assert!(
        utterances::split_features(&invalid, "sample.param", options(), ModelSource::Fresh)
            .is_err()
    );
    let mut invalid = features();
    invalid.frames = 0;
    invalid.values.clear();
    assert!(
        utterances::split_features(&invalid, "sample.param", options(), ModelSource::Fresh)
            .is_err()
    );
    let mut model =
        Model::read_from(include_bytes!("fixtures/utterances-c-trained.hsmm").as_slice()).unwrap();
    model.streams[0].mixtures[0].dimensions = 12;
    assert!(
        utterances::split_features(
            &features(),
            "sample.param",
            options(),
            ModelSource::Trained(&model)
        )
        .is_err()
    );
}
