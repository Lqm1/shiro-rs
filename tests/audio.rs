use ciglet_rs::{
    resampling::BoundaryPolicy,
    wave::{Encoding, Wave},
};
use shiro_rs::audio::{self, AudioOptions};
fn wave(samples: Vec<f32>) -> Wave<f32> {
    Wave {
        sample_rate: 8000,
        bits_per_sample: 32,
        channels: 1,
        encoding: Encoding::Float,
        samples,
    }
}
#[test]
fn normalization_dither_and_resampling_order() {
    let mut draws = [0.0, 0.5, 1.0].into_iter();
    let result = audio::prepare(
        wave(vec![-0.25, 0.5, 0.0]),
        AudioOptions {
            normalize: true,
            dither_level: 0.125,
            ..Default::default()
        },
        || draws.next().unwrap(),
    )
    .unwrap();
    assert_eq!(result.samples, [-0.625, 1.0, 0.125]);
    assert_eq!(result.sample_rate, 8000);
    assert!(draws.next().is_none());
    let normalized = audio::prepare(
        wave(vec![0.0, -0.0]),
        AudioOptions {
            normalize: true,
            ..Default::default()
        },
        || panic!("no dither draws expected"),
    )
    .unwrap();
    assert!(normalized.samples.iter().all(|x| x.is_finite()));
    assert_eq!(normalized.samples[1].to_bits(), (-0.0f32).to_bits());
    let changed = audio::prepare(
        wave(vec![0.75]),
        AudioOptions {
            output_sample_rate: Some(16000),
            ..Default::default()
        },
        || panic!(),
    )
    .unwrap();
    assert_eq!(changed.sample_rate, 16000);
    assert_eq!(changed.samples[0], 0.75);
    assert_eq!(
        audio::prepare(
            wave(vec![0.75]),
            AudioOptions {
                output_sample_rate: Some(16000),
                boundary: BoundaryPolicy::LegacySkipFirst,
                ..Default::default()
            },
            || panic!()
        )
        .unwrap()
        .samples,
        [0.0, 0.0]
    );
    assert!(
        audio::prepare(
            wave(vec![]),
            AudioOptions {
                normalize: true,
                ..Default::default()
            },
            || panic!()
        )
        .unwrap()
        .samples
        .is_empty()
    );
    assert_eq!(
        audio::prepare(
            wave(vec![1.0]),
            AudioOptions {
                dither_level: -1.0,
                ..Default::default()
            },
            || panic!()
        )
        .unwrap()
        .samples,
        [1.0]
    );
}
#[test]
fn invalid_audio_parameters_and_draws_fail() {
    for rate in [0, u32::MAX] {
        assert!(
            audio::prepare(
                wave(vec![1.0]),
                AudioOptions {
                    output_sample_rate: Some(rate),
                    ..Default::default()
                },
                || 0.5
            )
            .is_err()
        );
    }
    assert!(audio::prepare(wave(vec![f32::NAN]), AudioOptions::default(), || 0.5).is_err());
    assert!(
        audio::prepare(
            wave(vec![1.0]),
            AudioOptions {
                dither_level: f32::NAN,
                ..Default::default()
            },
            || 0.5
        )
        .is_err()
    );
    for draw in [-0.1, 1.1, f32::NAN] {
        assert!(
            audio::prepare(
                wave(vec![1.0]),
                AudioOptions {
                    dither_level: 0.125,
                    ..Default::default()
                },
                || draw
            )
            .is_err()
        );
    }
    assert!(
        audio::prepare(
            wave(vec![f32::MAX]),
            AudioOptions {
                dither_level: f32::MAX,
                ..Default::default()
            },
            || 1.0
        )
        .is_err()
    );
    let mut invalid = wave(vec![1.0]);
    invalid.sample_rate = 0;
    assert!(audio::prepare(invalid, AudioOptions::default(), || 0.5).is_err());
}

#[test]
fn original_c_runtime_dither_sequences() {
    for (bytes, mut sequence) in [
        (
            include_bytes!("fixtures/c-dither-windows.bin").as_slice(),
            audio::DitherSequence::windows(),
        ),
        (
            include_bytes!("fixtures/c-dither-linux.bin").as_slice(),
            audio::DitherSequence::linux_gnu(),
        ),
    ] {
        assert_eq!(&bytes[..4], b"DTH1");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 64);
        let maximum = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let mut draws = Vec::new();
        for index in 0..64 {
            let offset = 12 + index * 8;
            let integer = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            let expected = f32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            assert_eq!(
                expected.to_bits(),
                (integer as f32 / maximum as f32).to_bits()
            );
            let draw = sequence.next_uniform();
            assert_eq!(draw.to_bits(), expected.to_bits());
            draws.push(draw);
        }
        let prepared = audio::prepare(
            wave(vec![0.0; 64]),
            AudioOptions {
                dither_level: 1.0,
                ..Default::default()
            },
            || draws.remove(0),
        )
        .unwrap();
        for (index, actual) in prepared.samples.iter().enumerate() {
            let offset = 12 + 64 * 8 + index * 4;
            let expected = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        assert_eq!(bytes.len(), 12 + 64 * 12);
    }
}
