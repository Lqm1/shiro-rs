use liblrhsmm_rs::{Model, Observation, ObservationStream, serial::ModelEncoding};
use shiro_rs::{
    alignment::{self, DurationMode, Options},
    batch::Preset,
    features,
    labels::{self, SegmentationDocument},
    phonemap, segmentation,
};

const WAVES: [&[u8]; 3] = [
    include_bytes!("../../../tests/fixtures/cmu-slt-arctic_a0001.wav"),
    include_bytes!("../../../tests/fixtures/cmu-slt-arctic_a0002.wav"),
    include_bytes!("../../../tests/fixtures/cmu-slt-arctic_a0003.wav"),
];
const PARAMETERS: [&[u8]; 3] = [
    include_bytes!("../../../tests/fixtures/cmu-slt-arctic_a0001.param"),
    include_bytes!("../../../tests/fixtures/cmu-slt-arctic_a0002.param"),
    include_bytes!("../../../tests/fixtures/cmu-slt-arctic_a0003.param"),
];

#[test]
fn real_speech_features_and_historical_model_inference_match_corrected_c() {
    let encoded = include_bytes!("../../../tests/fixtures/cmu-arctic-all-speakers.hsmm");
    let model = Model::read_from(encoded.as_slice()).unwrap();
    let mut roundtrip = Vec::new();
    model
        .write_with_encoding(&mut roundtrip, ModelEncoding::WithoutVarianceFloors)
        .unwrap();
    assert_eq!(roundtrip, encoded);
    let map = phonemap::create(
        include_str!("../../../tests/fixtures/cmu-arpabet-phoneset.csv"),
        &phonemap::Options {
            states_per_phone: 5,
            streams: 3,
            ..phonemap::Options::default()
        },
    )
    .unwrap();
    let references: [SegmentationDocument; 2] = [
        serde_json::from_str(include_str!("../../../tests/fixtures/cmu-slt-c-hmm.json")).unwrap(),
        serde_json::from_str(include_str!("../../../tests/fixtures/cmu-slt-c-hsmm.json")).unwrap(),
    ];
    for (index, row) in include_str!("../../../tests/fixtures/cmu-slt-index.csv")
        .lines()
        .enumerate()
    {
        let wave =
            ciglet_rs::wave::read::<f32, _>(&mut std::io::Cursor::new(WAVES[index]), 100_000)
                .unwrap();
        assert_eq!(wave.sample_rate, 16000);
        let extracted =
            features::extract(&wave.samples, Preset::Mfcc12Da16k.feature_options()).unwrap();
        assert_eq!(extracted.columns, 36);
        let expected: Vec<_> = PARAMETERS[index]
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
            .collect();
        assert_eq!(extracted.values.len(), expected.len());
        let maximum = extracted
            .values
            .iter()
            .zip(&expected)
            .map(|(a, b)| (a - b).abs() / b.abs().max(1.0))
            .fold(0.0f32, f32::max);
        eprintln!("CMU SLT case {index}: max normalized feature difference {maximum}");
        assert!(maximum <= 5e-5);
        let mut streams = Vec::new();
        for stream in 0..3 {
            let values = extracted
                .values
                .chunks_exact(36)
                .flat_map(|frame| frame[stream * 12..(stream + 1) * 12].iter().copied())
                .collect();
            streams.push(ObservationStream {
                dimensions: 12,
                values,
            });
        }
        let observation = Observation {
            frames: extracted.frames,
            streams,
        };
        let (_, phones) = row.split_once(',').unwrap();
        let phones: Vec<_> = std::iter::once("sil")
            .chain(phones.split_whitespace())
            .chain(std::iter::once("sil"))
            .map(str::to_owned)
            .collect();
        let initial = segmentation::initial(&phones, &map, extracted.frames).unwrap();
        let hmm = alignment::align_states(
            &model,
            &observation,
            &initial,
            Options {
                duration_mode: DurationMode::Geometric,
                ..Options::default()
            },
        )
        .unwrap();
        let mut hsmm_options = Options::default();
        hsmm_options.hsmm.state_radius = 10.0;
        hsmm_options.hsmm.duration_extra = 50;
        let hsmm = alignment::align_states(&model, &observation, &hmm, hsmm_options).unwrap();
        for (mode, actual) in [&hmm, &hsmm].into_iter().enumerate() {
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(&references[mode].files[index].states).unwrap()
            );
        }
        let labels = labels::from_states(&hsmm, 0.005, false).unwrap();
        assert_eq!(labels.len(), phones.len());
        assert_eq!(labels.first().unwrap().start, 0.0);
        assert_eq!(labels.last().unwrap().end, extracted.frames as f64 * 0.005);
    }
}

#[test]
fn real_speech_initialization_and_both_training_modes_survive_model_reload() {
    use liblrhsmm_rs::Dataset;
    use shiro_rs::{dataset, initialization, training};

    let map = phonemap::create(
        include_str!("../../../tests/fixtures/cmu-arpabet-phoneset.csv"),
        &phonemap::Options {
            states_per_phone: 5,
            streams: 3,
            ..phonemap::Options::default()
        },
    )
    .unwrap();
    let uninitialized = phonemap::to_definition(&map, 12, 0.005)
        .unwrap()
        .build()
        .unwrap();
    let mut corpus = Dataset {
        observations: Vec::new(),
        segmentations: Vec::new(),
    };
    let mut states = Vec::new();
    for (index, row) in include_str!("../../../tests/fixtures/cmu-slt-index.csv")
        .lines()
        .enumerate()
    {
        let wave =
            ciglet_rs::wave::read::<f32, _>(&mut std::io::Cursor::new(WAVES[index]), 100_000)
                .unwrap();
        let extracted =
            features::extract(&wave.samples, Preset::Mfcc12Da16k.feature_options()).unwrap();
        let bytes: Vec<_> = extracted
            .values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        let observation =
            dataset::read_observation(bytes.as_slice(), &[12, 12, 12], 100_000).unwrap();
        let (_, phones) = row.split_once(',').unwrap();
        let phones: Vec<_> = std::iter::once("sil")
            .chain(phones.split_whitespace())
            .chain(std::iter::once("sil"))
            .map(str::to_owned)
            .collect();
        let initial = segmentation::initial(&phones, &map, extracted.frames).unwrap();
        corpus
            .segmentations
            .push(dataset::read_segmentation(&initial, &uninitialized).unwrap());
        corpus.observations.push(observation);
        states.push(initial);
    }
    let initialized = initialization::initialize(
        &uninitialized,
        &corpus,
        initialization::Options {
            flat_start: true,
            globally_tied: true,
            variance_floor_ratio: 1.0,
        },
    )
    .unwrap();
    let mut initial_bytes = Vec::new();
    initialized.write_to(&mut initial_bytes).unwrap();
    let files: Vec<_> = corpus
        .observations
        .iter()
        .zip(&corpus.segmentations)
        .map(|(observation, segmentation)| Dataset {
            observations: vec![observation.clone()],
            segmentations: vec![segmentation.clone()],
        })
        .collect();
    for duration_mode in [
        liblrhsmm_rs::DurationMode::Geometric,
        liblrhsmm_rs::DurationMode::Normal,
    ] {
        let mut options = training::Options {
            iterations: 2,
            duration_mode,
            deterministic_annealing: true,
            termination_threshold: 0.0,
            ..training::Options::default()
        };
        options.geometric.pruning_slope = 0.8;
        let trained = training::train(&initialized, &files, options).unwrap();
        assert_eq!(trained.iterations.len(), 2);
        for report in &trained.iterations {
            assert!(report.mean_log_likelihood.is_finite());
            assert_eq!(report.file_likelihoods.len(), 3);
            assert!(
                report
                    .file_likelihoods
                    .iter()
                    .all(|row| row.len() == 1 && row[0].is_finite())
            );
        }
        let mut encoded = Vec::new();
        trained.model.write_to(&mut encoded).unwrap();
        assert_ne!(encoded, initial_bytes);
        let reloaded = Model::read_from(encoded.as_slice()).unwrap();
        let mut saved_again = Vec::new();
        reloaded.write_to(&mut saved_again).unwrap();
        assert_eq!(saved_again, encoded);
        let mut inference = Options::default();
        inference.hsmm.state_radius = 10.0;
        inference.hsmm.duration_extra = 50;
        for (observation, initial) in corpus.observations.iter().zip(&states) {
            let before =
                alignment::align_states(&trained.model, observation, initial, inference).unwrap();
            let after =
                alignment::align_states(&reloaded, observation, initial, inference).unwrap();
            assert_eq!(
                serde_json::to_value(&before).unwrap(),
                serde_json::to_value(&after).unwrap()
            );
            let labels = labels::from_states(&after, 0.005, false).unwrap();
            assert!(!labels.is_empty());
            assert_eq!(
                labels.last().unwrap().end,
                observation.frames as f64 * 0.005
            );
        }
    }
    let mut unchanged = Vec::new();
    initialized.write_to(&mut unchanged).unwrap();
    assert_eq!(unchanged, initial_bytes);
}
