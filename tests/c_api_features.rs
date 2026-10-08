#![cfg(feature = "c-api")]
use shiro_rs::{
    c_api::*,
    features::{self, Energy, FeatureKind, FeatureOptions},
};
use std::ptr::{null, null_mut};

struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn bytes(&mut self, count: usize) -> &[u8] {
        let (head, tail) = self.0.split_at(count);
        self.0 = tail;
        head
    }
    fn integer(&mut self) -> usize {
        u32::from_le_bytes(self.bytes(4).try_into().unwrap()) as usize
    }
    fn scalar(&mut self) -> f32 {
        f32::from_le_bytes(self.bytes(4).try_into().unwrap())
    }
}
unsafe fn array(values: &[f32]) -> *mut ShiroRsArrayF32 {
    let mut owner = null_mut();
    // SAFETY: Readable complete input and independent output slot.
    assert_eq!(
        unsafe { shiro_rs_array_f32_create(values.as_ptr(), values.len(), &mut owner) },
        0
    );
    owner
}
unsafe fn values(owner: *const ShiroRsArrayF32) -> Vec<f32> {
    // SAFETY: Live readable owner and independent complete output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_array_f32_length(owner, &mut count), 0);
        let mut result = vec![0.0; count];
        assert_eq!(
            shiro_rs_array_f32_copy(owner, 0, result.as_mut_ptr(), count),
            0
        );
        result
    }
}
unsafe fn extracted(
    input: &[f32],
    options: ShiroRsFeatureOptions,
) -> (ShiroRsFeatureInfo, Vec<f32>) {
    // SAFETY: Unique live owners and independent output slots.
    unsafe {
        let mut input = array(input);
        let mut features = null_mut();
        assert_eq!(shiro_rs_features_extract(input, &options, &mut features), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut input), 0);
        let mut clone = null_mut();
        assert_eq!(shiro_rs_features_clone(features, &mut clone), 0);
        assert_eq!(shiro_rs_features_release(&mut features), 0);
        let mut info = ShiroRsFeatureInfo {
            frames: 0,
            columns: 0,
        };
        let mut samples = null_mut();
        assert_eq!(shiro_rs_features_get_info(clone, &mut info), 0);
        assert_eq!(shiro_rs_features_get_values(clone, &mut samples), 0);
        assert_eq!(shiro_rs_features_release(&mut clone), 0);
        let result = values(samples);
        assert_eq!(shiro_rs_array_f32_release(&mut samples), 0);
        (info, result)
    }
}
fn close(actual: f32, expected: f32) {
    if expected.is_nan() {
        assert!(actual.is_nan());
    } else if expected.is_infinite() {
        assert_eq!(actual, expected);
    } else {
        assert!(
            (f64::from(actual) - f64::from(expected)).abs() / f64::from(expected).abs().max(1.0)
                < 2e-5,
            "{actual} vs {expected}"
        );
    }
}

#[test]
fn all_seventy_two_original_c_records_retain_complete_matrices() {
    let mut reader = Reader(include_bytes!("fixtures/c-xxcc.bin"));
    assert_eq!(reader.bytes(4), b"XCC1");
    assert_eq!(reader.integer(), 72);
    for _ in 0..72 {
        let kind = reader.integer() as u32;
        let energy = reader.integer() as u32;
        let flags = reader.integer();
        let frame_length = reader.integer();
        let hop = reader.scalar();
        let count = reader.integer();
        let input: Vec<f32> = (0..count).map(|_| reader.scalar()).collect();
        let frames = reader.integer();
        let columns = reader.integer();
        let options = ShiroRsFeatureOptions {
            kind,
            energy,
            frame_length,
            hop,
            channels: 12,
            sample_rate_hz: 16000.0,
            warp: 0.85,
            include_dc: (flags & 1) as u32,
            delta: ((flags >> 1) & 1) as u32,
            acceleration: ((flags >> 2) & 1) as u32,
            ..FeatureOptions::default().into()
        };
        // SAFETY: Helper manages uniquely owned inputs/outputs.
        let (info, actual) = unsafe { extracted(&input, options) };
        assert_eq!(info, ShiroRsFeatureInfo { frames, columns });
        assert_eq!(actual.len(), frames * columns);
        for value in actual {
            close(value, reader.scalar());
        }
    }
    assert!(reader.0.is_empty());
}

#[test]
fn every_native_setting_and_empty_fractional_frames_match_independent_native_calls() {
    let input: Vec<f32> = (0..127).map(|index| (index as f32 * 0.23).sin()).collect();
    for kind in [FeatureKind::Mfcc, FeatureKind::Mfbe, FeatureKind::Plpcc] {
        for energy in [None, Some(Energy::Rms), Some(Energy::Decibels)] {
            let options = FeatureOptions {
                kind,
                energy,
                order: 5,
                channels: 9,
                frame_length: 63,
                hop: 17.5,
                sample_rate_hz: 22050.0,
                minimum_bandwidth_hz: 275.0,
                warp: 1.25,
                include_dc: true,
                delta: true,
                acceleration: true,
            };
            for signal in [input.as_slice(), &[]] {
                let expected = features::extract(signal, options).unwrap();
                // SAFETY: Helper owns and releases every input/output independently.
                let (info, actual) = unsafe { extracted(signal, options.into()) };
                assert_eq!(
                    (info.frames, info.columns),
                    (expected.frames, expected.columns)
                );
                assert_eq!(
                    actual.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                    expected
                        .values
                        .iter()
                        .map(|v| v.to_bits())
                        .collect::<Vec<_>>()
                );
            }
        }
    }
    // SAFETY: Independent initialized output descriptor.
    unsafe {
        let mut defaults: ShiroRsFeatureOptions = FeatureOptions::default().into();
        defaults.order = 99;
        assert_eq!(shiro_rs_feature_options_default(&mut defaults), 0);
        assert_eq!(defaults, FeatureOptions::default().into());
    }
}

#[test]
fn arbitrary_public_fields_float_bits_and_failed_outputs_retain_independent_owners() {
    // SAFETY: Live owners and independent outputs, invalid nulls rejected before access.
    unsafe {
        let bits = [0x8000_0000, 1, 0x7fc1_2345, 0x7f80_0000];
        let mut source = array(&bits.map(f32::from_bits));
        let mut owner = null_mut();
        assert_eq!(
            shiro_rs_features_create(usize::MAX, usize::MAX - 1, source, &mut owner),
            0
        );
        let mut clone = null_mut();
        assert_eq!(shiro_rs_features_clone(owner, &mut clone), 0);
        assert_eq!(shiro_rs_features_release(&mut owner), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut source), 0);
        let mut info = ShiroRsFeatureInfo {
            frames: 0,
            columns: 0,
        };
        assert_eq!(shiro_rs_features_get_info(clone, &mut info), 0);
        assert_eq!((info.frames, info.columns), (usize::MAX, usize::MAX - 1));
        let mut snapshot = null_mut();
        assert_eq!(shiro_rs_features_get_values(clone, &mut snapshot), 0);
        let retained = clone;
        for field in 0..5 {
            let mut options: ShiroRsFeatureOptions = FeatureOptions::default().into();
            match field {
                0 => options.kind = 3,
                1 => options.energy = 3,
                2 => options.include_dc = 2,
                3 => options.delta = 2,
                _ => options.acceleration = 2,
            }
            source = array(&[]);
            assert_eq!(shiro_rs_features_extract(source, &options, &mut clone), 2);
            assert_eq!(clone, retained);
            assert_eq!(shiro_rs_array_f32_release(&mut source), 0);
        }
        source = array(&[f32::NAN]);
        let options: ShiroRsFeatureOptions = FeatureOptions::default().into();
        assert_eq!(shiro_rs_features_extract(source, &options, &mut clone), 3);
        assert_eq!(clone, retained);
        assert_eq!(shiro_rs_features_extract(source, null(), &mut clone), 1);
        assert_eq!(clone, retained);
        assert_eq!(shiro_rs_features_create(0, 0, null(), &mut clone), 1);
        assert_eq!(clone, retained);
        assert_eq!(shiro_rs_features_get_info(null(), &mut info), 1);
        assert_eq!(info.frames, usize::MAX);
        assert_eq!(shiro_rs_features_get_values(clone, null_mut()), 1);
        assert_eq!(shiro_rs_features_clone(clone, null_mut()), 1);
        assert_eq!(shiro_rs_feature_options_default(null_mut()), 1);
        assert_eq!(shiro_rs_features_release(null_mut()), 1);
        assert_eq!(shiro_rs_features_release(&mut clone), 0);
        assert_eq!(
            values(snapshot)
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            bits
        );
        assert_eq!(shiro_rs_array_f32_release(&mut snapshot), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut source), 0);
        assert_eq!(shiro_rs_features_release(&mut clone), 0);
    }
}
