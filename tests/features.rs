use shiro_rs::features::{Energy, FeatureKind, FeatureOptions, extract};
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn bytes(&mut self, n: usize) -> &[u8] {
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        a
    }
    fn integer(&mut self) -> usize {
        u32::from_le_bytes(self.bytes(4).try_into().unwrap()) as usize
    }
    fn scalar(&mut self) -> f32 {
        f32::from_le_bytes(self.bytes(4).try_into().unwrap())
    }
}
#[test]
fn original_xxcc_pipeline_matches_c() {
    let mut reader = Reader(include_bytes!("fixtures/c-xxcc.bin"));
    assert_eq!(reader.bytes(4), b"XCC1");
    assert_eq!(reader.integer(), 72);
    let mut maximum = 0.0f64;
    let mut count = 0;
    let mut nonfinite = 0;
    for record in 0..72 {
        let kind = [FeatureKind::Mfcc, FeatureKind::Mfbe, FeatureKind::Plpcc][reader.integer()];
        let energy = [None, Some(Energy::Rms), Some(Energy::Decibels)][reader.integer()];
        let flags = reader.integer();
        let frame_length = reader.integer();
        let hop = reader.scalar();
        let samples = reader.integer();
        let input: Vec<f32> = (0..samples).map(|_| reader.scalar()).collect();
        let expected_frames = reader.integer();
        let expected_columns = reader.integer();
        let options = FeatureOptions {
            kind,
            energy,
            frame_length,
            hop,
            channels: 12,
            sample_rate_hz: 16000.0,
            warp: 0.85,
            include_dc: flags & 1 != 0,
            delta: flags & 2 != 0,
            acceleration: flags & 4 != 0,
            ..Default::default()
        };
        let result = extract(&input, options).unwrap();
        assert_eq!(result.frames, expected_frames);
        assert_eq!(result.columns, expected_columns);
        assert_eq!(result.values.len(), result.frames * result.columns);
        for (index, &actual) in result.values.iter().enumerate() {
            let expected = reader.scalar();
            count += 1;
            if !expected.is_finite() {
                if expected.is_nan() {
                    assert!(actual.is_nan(), "NaN {record}/{index}");
                } else {
                    assert_eq!(actual, expected);
                }
                nonfinite += 1;
                continue;
            }
            let error = (f64::from(actual) - f64::from(expected)).abs()
                / f64::from(expected).abs().max(1.0);
            maximum = maximum.max(error);
            assert!(
                error < 2e-5,
                "xxcc {record}/{index}: {actual} vs {expected}, {error:e}"
            );
        }
    }
    assert!(reader.0.is_empty());
    eprintln!(
        "Original xxcc: {count} outputs, normalized error {maximum:e}, {nonfinite} matching nonfinite energy/difference outputs"
    );
}
#[test]
fn dimensions_validation_and_endpoint_energy_correction() {
    let options = FeatureOptions {
        channels: 4,
        sample_rate_hz: 16000.0,
        frame_length: 512,
        hop: 256.0,
        energy: Some(Energy::Rms),
        ..Default::default()
    };
    let mut signal = vec![0.0; 512];
    signal[0] = 1.0;
    let result = extract(&signal, options).unwrap();
    assert_eq!(result.frames, 2);
    assert_eq!(result.columns, 13);
    assert!(result.values[12] > 0.0);
    assert_eq!(result.values[25], 0.0);
    let no_frames = extract(&[], options).unwrap();
    assert_eq!(no_frames.frames, 0);
    assert!(no_frames.values.is_empty());
    assert!(
        extract(
            &signal,
            FeatureOptions {
                hop: 0.0,
                ..options
            }
        )
        .is_err()
    );
    assert!(
        extract(
            &signal,
            FeatureOptions {
                sample_rate_hz: -1.0,
                ..options
            }
        )
        .is_err()
    );
    assert!(
        extract(
            &signal,
            FeatureOptions {
                warp: 0.0,
                ..options
            }
        )
        .is_err()
    );
    assert!(
        extract(
            &signal,
            FeatureOptions {
                frame_length: 0,
                ..options
            }
        )
        .is_err()
    );
    assert!(extract(&[f32::NAN], options).is_err());
    let mfbe = extract(
        &signal,
        FeatureOptions {
            kind: FeatureKind::Mfbe,
            include_dc: true,
            ..options
        },
    )
    .unwrap();
    assert_eq!(mfbe.columns, 14);
    assert_eq!(mfbe.values[12], 0.0);
}
