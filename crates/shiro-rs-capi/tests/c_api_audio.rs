use shiro_rs::audio::{self, AudioOptions};
use shiro_rs::dsp::{
    resampling::{BoundaryPolicy, KernelPolicy},
    wave::{Encoding, Wave},
};
use shiro_rs_capi::*;
use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};

unsafe fn array(values: &[f32]) -> *mut ShiroRsArrayF32 {
    let mut owner = null_mut();
    // SAFETY: Readable input and independent output.
    assert_eq!(
        unsafe { shiro_rs_array_f32_create(values.as_ptr(), values.len(), &mut owner) },
        0
    );
    owner
}
unsafe fn bytes(values: &[u8]) -> *mut ShiroRsBytes {
    let mut owner = null_mut();
    // SAFETY: Readable input and independent output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(values.as_ptr(), values.len(), &mut owner) },
        0
    );
    owner
}
unsafe fn samples(owner: *const ShiroRsArrayF32) -> Vec<f32> {
    // SAFETY: Live input owner and independent complete output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_array_f32_length(owner, &mut count), 0);
        let mut output = vec![0.0; count];
        assert_eq!(
            shiro_rs_array_f32_copy(owner, 0, output.as_mut_ptr(), count),
            0
        );
        output
    }
}
unsafe fn snapshot(mut owner: *mut ShiroRsAudio) -> (u32, Vec<f32>) {
    // SAFETY: Unique input owner, independent clone/snapshot buffers.
    unsafe {
        let mut clone = null_mut();
        assert_eq!(shiro_rs_audio_clone(owner, &mut clone), 0);
        assert_eq!(shiro_rs_audio_release(&mut owner), 0);
        let mut rate = 0;
        let mut values = null_mut();
        assert_eq!(shiro_rs_audio_sample_rate(clone, &mut rate), 0);
        assert_eq!(shiro_rs_audio_get_samples(clone, &mut values), 0);
        assert_eq!(shiro_rs_audio_release(&mut clone), 0);
        let output = samples(values);
        assert_eq!(shiro_rs_array_f32_release(&mut values), 0);
        (rate, output)
    }
}
unsafe extern "C" fn sequence(context: *mut c_void, output: *mut f32) -> u32 {
    // SAFETY: Test provides exclusive live sequence as context and callback output.
    unsafe { shiro_rs_dither_next_uniform(context.cast(), output) }
}
struct Draws {
    values: Vec<f32>,
    position: usize,
    fail_at: Option<usize>,
}
unsafe extern "C" fn controlled(context: *mut c_void, output: *mut f32) -> u32 {
    // SAFETY: Synchronous exclusive live context and writable output.
    unsafe {
        let state = &mut *context.cast::<Draws>();
        let position = state.position;
        state.position += 1;
        if state.fail_at == Some(position) {
            return 7;
        }
        match state.values.get(position) {
            Some(&draw) => {
                output.write(draw);
                0
            }
            None => 8,
        }
    }
}

#[test]
fn all_original_c_wave_conversions_and_bounded_decode_match_complete_outputs() {
    let cases: &[(bool, Option<u32>, &[u8])] = &[
        (
            false,
            None,
            include_bytes!("../../../tests/fixtures/c-audio-input.plain.raw"),
        ),
        (
            true,
            None,
            include_bytes!("../../../tests/fixtures/c-audio-input.normalized.raw"),
        ),
        (
            false,
            Some(32000),
            include_bytes!("../../../tests/fixtures/c-audio-input.up.raw"),
        ),
        (
            false,
            Some(8000),
            include_bytes!("../../../tests/fixtures/c-audio-input.down.raw"),
        ),
        (
            true,
            Some(8000),
            include_bytes!("../../../tests/fixtures/c-audio-input.normalized-down.raw"),
        ),
    ];
    // SAFETY: Unique owners and independent input/options/output buffers.
    unsafe {
        let mut wire = bytes(include_bytes!("../../../tests/fixtures/c-audio-input.wav"));
        for &(normalize, output_sample_rate, expected) in cases {
            let options: ShiroRsAudioOptions = AudioOptions {
                normalize,
                output_sample_rate,
                boundary: BoundaryPolicy::LegacySkipFirst,
                kernel: KernelPolicy::Legacy,
                ..Default::default()
            }
            .into();
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_audio_prepare_wave_bytes(
                    wire,
                    1024,
                    &options,
                    None,
                    null_mut(),
                    &mut output
                ),
                0
            );
            let (rate, actual) = snapshot(output);
            assert_eq!(rate, output_sample_rate.unwrap_or(16000));
            assert_eq!(actual.len() * 4, expected.len());
            for (actual, expected) in actual.into_iter().zip(expected.chunks_exact(4)) {
                let expected = f32::from_le_bytes(expected.try_into().unwrap());
                assert!(
                    (f64::from(actual) - f64::from(expected)).abs()
                        / f64::from(expected).abs().max(1.0)
                        < 2e-7
                );
            }
        }
        let options: ShiroRsAudioOptions = AudioOptions::default().into();
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_audio_prepare_wave_bytes(wire, 1, &options, None, null_mut(), &mut output),
            3
        );
        assert!(output.is_null());
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
    }
}

#[test]
fn both_original_random_sequences_and_exact_signed_dither_values_match_c() {
    // SAFETY: Unique sequence/array/audio owners; synchronous borrowed RNG contexts.
    unsafe {
        for (linux, fixture) in [
            (
                false,
                include_bytes!("../../../tests/fixtures/c-dither-windows.bin").as_slice(),
            ),
            (
                true,
                include_bytes!("../../../tests/fixtures/c-dither-linux.bin").as_slice(),
            ),
        ] {
            let mut rng = null_mut();
            let constructor = if linux {
                shiro_rs_dither_linux_gnu
            } else {
                shiro_rs_dither_windows
            };
            assert_eq!(constructor(&mut rng), 0);
            assert_eq!(shiro_rs_dither_next_uniform(rng, null_mut()), 1);
            for index in 0..64 {
                let mut draw = -1.0;
                assert_eq!(shiro_rs_dither_next_uniform(rng, &mut draw), 0);
                let offset = 12 + index * 8 + 4;
                assert_eq!(
                    draw.to_bits(),
                    f32::from_le_bytes(fixture[offset..offset + 4].try_into().unwrap()).to_bits()
                );
            }
            assert_eq!(shiro_rs_dither_release(&mut rng), 0);
            assert_eq!(constructor(&mut rng), 0);
            let mut input = array(&[0.0; 64]);
            let header = ShiroRsWaveInfo {
                sample_rate: 8000,
                bits_per_sample: 32,
                channels: 1,
                encoding: 1,
            };
            let options: ShiroRsAudioOptions = AudioOptions {
                dither_level: 1.0,
                ..Default::default()
            }
            .into();
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_audio_prepare(
                    input,
                    &header,
                    &options,
                    Some(sequence),
                    rng.cast(),
                    &mut output
                ),
                0
            );
            let (rate, actual) = snapshot(output);
            assert_eq!(rate, 8000);
            for (index, actual) in actual.into_iter().enumerate() {
                let offset = 12 + 64 * 8 + index * 4;
                assert_eq!(
                    actual.to_bits(),
                    f32::from_le_bytes(fixture[offset..offset + 4].try_into().unwrap()).to_bits()
                );
            }
            assert_eq!(shiro_rs_array_f32_release(&mut input), 0);
            assert_eq!(shiro_rs_dither_release(&mut rng), 0);
            assert_eq!(shiro_rs_dither_release(&mut rng), 0);
        }
        let mut wire = bytes(include_bytes!("../../../tests/fixtures/c-audio-input.wav"));
        let mut rng = null_mut();
        assert_eq!(shiro_rs_dither_linux_gnu(&mut rng), 0);
        let options: ShiroRsAudioOptions = AudioOptions {
            dither_level: 0.125,
            ..Default::default()
        }
        .into();
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_audio_prepare_wave_bytes(
                wire,
                1024,
                &options,
                Some(sequence),
                rng.cast(),
                &mut output
            ),
            0
        );
        let (_, actual) = snapshot(output);
        let expected = include_bytes!("../../../tests/fixtures/c-audio-input.dither-linux.raw");
        assert_eq!(
            actual
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(shiro_rs_dither_release(&mut rng), 0);
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
    }
}

#[test]
fn all_native_options_header_fields_and_callback_consumption_match_independent_calls() {
    // SAFETY: Unique owners/descriptors and synchronous exclusive controlled draw context.
    unsafe {
        for boundary in [
            BoundaryPolicy::IncludeFirst,
            BoundaryPolicy::LegacySkipFirst,
        ] {
            for kernel in [KernelPolicy::Stable, KernelPolicy::Legacy] {
                for rate in [None, Some(16000)] {
                    let wave = Wave {
                        sample_rate: 8000,
                        bits_per_sample: 0,
                        channels: u16::MAX,
                        encoding: Encoding::Pcm,
                        samples: vec![-0.25, 0.5, 0.0],
                    };
                    let options = AudioOptions {
                        normalize: true,
                        dither_level: 0.125,
                        output_sample_rate: rate,
                        boundary,
                        kernel,
                    };
                    let mut draws = [0.0, 0.5, 1.0].into_iter();
                    let expected =
                        audio::prepare(wave.clone(), options, || draws.next().unwrap()).unwrap();
                    let header = ShiroRsWaveInfo {
                        sample_rate: wave.sample_rate,
                        bits_per_sample: wave.bits_per_sample,
                        channels: wave.channels,
                        encoding: 0,
                    };
                    let mut input = array(&wave.samples);
                    let mut context = Draws {
                        values: vec![0.0, 0.5, 1.0],
                        position: 0,
                        fail_at: None,
                    };
                    let mut output = null_mut();
                    assert_eq!(
                        shiro_rs_audio_prepare(
                            input,
                            &header,
                            &options.into(),
                            Some(controlled),
                            (&mut context as *mut Draws).cast(),
                            &mut output
                        ),
                        0
                    );
                    assert_eq!(context.position, 3);
                    let (actual_rate, actual) = snapshot(output);
                    assert_eq!(actual_rate, expected.sample_rate);
                    assert_eq!(
                        actual.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                        expected
                            .samples
                            .iter()
                            .map(|v| v.to_bits())
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(samples(input), wave.samples);
                    assert_eq!(shiro_rs_array_f32_release(&mut input), 0);
                }
            }
        }
        let mut defaults: ShiroRsAudioOptions = AudioOptions::default().into();
        defaults.normalize = 99;
        assert_eq!(shiro_rs_audio_options_default(&mut defaults), 0);
        assert_eq!(defaults, AudioOptions::default().into());
    }
}

#[test]
fn arbitrary_audio_bits_optional_rate_and_failed_callback_outputs_are_preserved() {
    // SAFETY: Unique live owners and independent output slots; invalid descriptors
    // are rejected before use. Draw state remains exclusive during callback.
    unsafe {
        let bits = [0x8000_0000, 1, 0x7fc1_2345, 0x7f80_0000];
        let mut source = array(&bits.map(f32::from_bits));
        let mut owner = null_mut();
        assert_eq!(shiro_rs_audio_create(u32::MAX, source, &mut owner), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut source), 0);
        let mut retained = owner;
        let header = ShiroRsWaveInfo {
            sample_rate: 8000,
            bits_per_sample: 0,
            channels: 0,
            encoding: 1,
        };
        source = array(&[0.0, 0.5, 1.0]);
        let base: ShiroRsAudioOptions = AudioOptions::default().into();
        for field in 0..4 {
            let mut options = base;
            match field {
                0 => options.normalize = 2,
                1 => options.has_output_sample_rate = 2,
                2 => options.boundary = 2,
                _ => options.kernel = 2,
            }
            assert_eq!(
                shiro_rs_audio_prepare(source, &header, &options, None, null_mut(), &mut retained),
                2
            );
            assert_eq!(retained, owner);
        }
        let mut options = base;
        options.has_output_sample_rate = 1;
        assert_eq!(
            shiro_rs_audio_prepare(source, &header, &options, None, null_mut(), &mut retained),
            3
        );
        assert_eq!(retained, owner);
        options.has_output_sample_rate = 0;
        options.output_sample_rate = u32::MAX;
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_audio_prepare(source, &header, &options, None, null_mut(), &mut output),
            0
        );
        assert_eq!(snapshot(output), (8000, vec![0.0, 0.5, 1.0]));
        options.dither_level = 0.125;
        let mut context = Draws {
            values: vec![0.0, 0.5, 1.0],
            position: 0,
            fail_at: Some(1),
        };
        assert_eq!(
            shiro_rs_audio_prepare(
                source,
                &header,
                &options,
                Some(controlled),
                (&mut context as *mut Draws).cast(),
                &mut retained
            ),
            3
        );
        assert_eq!(context.position, 2);
        assert_eq!(retained, owner);
        assert_eq!(
            shiro_rs_audio_prepare(source, &header, &options, None, null_mut(), &mut retained),
            3
        );
        assert_eq!(retained, owner);
        assert_eq!(
            shiro_rs_audio_prepare(source, null(), &options, None, null_mut(), &mut retained),
            1
        );
        assert_eq!(retained, owner);
        options.dither_level = -1.0;
        context.position = 0;
        assert_eq!(
            shiro_rs_audio_prepare(
                source,
                &header,
                &options,
                Some(controlled),
                (&mut context as *mut Draws).cast(),
                &mut output
            ),
            0
        );
        assert_eq!(context.position, 0);
        assert_eq!(shiro_rs_audio_release(&mut output), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut source), 0);
        let mut rate = 42;
        assert_eq!(shiro_rs_audio_sample_rate(null(), &mut rate), 1);
        assert_eq!(rate, 42);
        assert_eq!(shiro_rs_audio_options_default(null_mut()), 1);
        assert_eq!(shiro_rs_audio_get_samples(owner, null_mut()), 1);
        assert_eq!(shiro_rs_audio_clone(owner, null_mut()), 1);
        assert_eq!(shiro_rs_audio_create(0, null(), &mut retained), 1);
        assert_eq!(retained, owner);
        assert_eq!(shiro_rs_audio_release(null_mut()), 1);
        assert_eq!(shiro_rs_dither_windows(null_mut()), 1);
        assert_eq!(shiro_rs_dither_linux_gnu(null_mut()), 1);
        assert_eq!(shiro_rs_dither_release(null_mut()), 1);
        let (rate, actual) = snapshot(owner);
        owner = null_mut();
        assert_eq!(rate, u32::MAX);
        assert_eq!(actual.iter().map(|v| v.to_bits()).collect::<Vec<_>>(), bits);
        assert_eq!(shiro_rs_audio_release(&mut owner), 0);
    }
}
