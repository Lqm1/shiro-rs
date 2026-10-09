use shiro_rs::{
    features::Features,
    hsmm::Model,
    utterances::{self, ModelSource, Options, SegmentedUtterances},
};
use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

fn features() -> Features {
    Features {
        frames: 40,
        columns: 13,
        values: include_bytes!("../../../tests/fixtures/utterances-c-features.bin")
            .chunks_exact(4)
            .map(|value| f32::from_le_bytes(value.try_into().unwrap()))
            .collect(),
    }
}
fn wire(model: &Model) -> Vec<u8> {
    let mut output = Vec::new();
    model.write_to(&mut output).unwrap();
    output
}
unsafe fn owned(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable input range and independent output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn copied(data: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live immutable owner and independent initialized output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(data, &mut count), 0);
        let mut output = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(data, 0, output.as_mut_ptr(), count), 0);
        output
    }
}
unsafe fn model_wire(model: *const ShiroRsModel) -> Vec<u8> {
    // SAFETY: Live immutable owner and unique output.
    unsafe {
        let mut output = null_mut();
        assert_eq!(shiro_rs_model_write_bytes(model, 0, &mut output), 0);
        let bytes = copied(output);
        assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        bytes
    }
}
unsafe fn json<T>(
    owner: *const T,
    write: unsafe extern "C" fn(*const T, *mut *mut ShiroRsBytes) -> u32,
) -> serde_json::Value {
    // SAFETY: Live immutable owner and independent output.
    unsafe {
        let mut output = null_mut();
        assert_eq!(write(owner, &mut output), 0);
        let value = serde_json::from_slice(&copied(output)).unwrap();
        assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        value
    }
}
unsafe fn feature_owner(native: &Features) -> *mut ShiroRsFeatures {
    // SAFETY: Complete readable samples and independent owned output.
    unsafe {
        let mut values = null_mut();
        assert_eq!(
            shiro_rs_array_f32_create(native.values.as_ptr(), native.values.len(), &mut values),
            0
        );
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_features_create(native.frames, native.columns, values, &mut output),
            0
        );
        assert_eq!(shiro_rs_array_f32_release(&mut values), 0);
        output
    }
}
unsafe fn model_owner(native: &Model) -> *mut ShiroRsModel {
    // SAFETY: Independent input/output owners.
    unsafe {
        let mut bytes = owned(&wire(native));
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_model_read_bytes(bytes, 16 * 1024 * 1024, &mut output),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        output
    }
}

unsafe fn compare(owner: *const ShiroRsUtterances, expected: &SegmentedUtterances) {
    // SAFETY: Immutable result, independent snapshots and unique releases.
    unsafe {
        let (
            mut map,
            mut definition,
            mut phones,
            mut initial,
            mut alignment,
            mut reports,
            mut labels,
        ) = (
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
        );
        assert_eq!(shiro_rs_utterances_get_phonemap(owner, &mut map), 0);
        assert_eq!(
            json(map, shiro_rs_phone_map_write_json),
            serde_json::to_value(&expected.phonemap).unwrap()
        );
        assert_eq!(
            shiro_rs_utterances_get_definition(owner, &mut definition),
            0
        );
        assert_eq!(
            json(definition, shiro_rs_definition_write_json),
            serde_json::to_value(&expected.definition).unwrap()
        );
        assert_eq!(shiro_rs_utterances_get_phones(owner, &mut phones), 0);
        let mut count = 0;
        assert_eq!(shiro_rs_strings_length(phones, &mut count), 0);
        assert_eq!(count, expected.phones.len());
        for (index, expected) in expected.phones.iter().enumerate() {
            let mut output = null_mut();
            assert_eq!(shiro_rs_strings_get(phones, index, &mut output), 0);
            assert_eq!(copied(output), expected.as_bytes());
            assert_eq!(shiro_rs_bytes_release(&mut output), 0);
        }
        assert_eq!(
            shiro_rs_utterances_get_initial_segmentation(owner, &mut initial),
            0
        );
        assert_eq!(
            json(initial, shiro_rs_document_write_json),
            serde_json::to_value(&expected.initial_segmentation).unwrap()
        );
        assert_eq!(shiro_rs_utterances_get_alignment(owner, &mut alignment), 0);
        assert_eq!(
            json(alignment, shiro_rs_document_write_json),
            serde_json::to_value(&expected.alignment).unwrap()
        );
        for (stage, expected) in [
            (0, expected.uninitialized_model.as_ref()),
            (1, expected.initialized_model.as_ref()),
            (2, Some(&expected.model)),
        ] {
            let mut present = 99;
            assert_eq!(shiro_rs_utterances_has_model(owner, stage, &mut present), 0);
            assert_eq!(present, u32::from(expected.is_some()));
            let mut model = null_mut();
            assert_eq!(
                shiro_rs_utterances_get_model(owner, stage, &mut model),
                if present == 1 { 0 } else { 2 }
            );
            if let Some(expected) = expected {
                assert_eq!(model_wire(model), wire(expected));
                assert_eq!(shiro_rs_model_release(&mut model), 0);
            } else {
                assert!(model.is_null());
            }
        }
        assert_eq!(shiro_rs_utterances_get_iterations(owner, &mut reports), 0);
        assert_eq!(shiro_rs_iteration_reports_length(reports, &mut count), 0);
        assert_eq!(count, expected.iterations.len());
        for (index, expected) in expected.iterations.iter().enumerate() {
            let mut report = null_mut();
            assert_eq!(
                shiro_rs_iteration_reports_get(reports, index, &mut report),
                0
            );
            let mut info = ShiroRsIterationInfo {
                iteration: 0,
                temperature: 0.0,
                mean_log_likelihood: 0.0,
            };
            assert_eq!(shiro_rs_iteration_report_info(report, &mut info), 0);
            assert_eq!(
                (
                    info.iteration,
                    info.temperature.to_bits(),
                    info.mean_log_likelihood.to_bits()
                ),
                (
                    expected.iteration,
                    expected.temperature.to_bits(),
                    expected.mean_log_likelihood.to_bits()
                )
            );
            assert_eq!(shiro_rs_iteration_report_file_count(report, &mut count), 0);
            assert_eq!(count, expected.file_likelihoods.len());
            for (index, expected) in expected.file_likelihoods.iter().enumerate() {
                let mut row = null_mut();
                assert_eq!(
                    shiro_rs_iteration_report_get_file(report, index, &mut row),
                    0
                );
                assert_eq!(shiro_rs_array_f32_length(row, &mut count), 0);
                assert_eq!(count, expected.len());
                let mut actual = vec![0.0; count];
                assert_eq!(
                    shiro_rs_array_f32_copy(row, 0, actual.as_mut_ptr(), count),
                    0
                );
                assert_eq!(
                    actual
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>(),
                    expected
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>()
                );
                assert_eq!(shiro_rs_array_f32_release(&mut row), 0);
            }
            assert_eq!(shiro_rs_iteration_report_release(&mut report), 0);
        }
        assert_eq!(shiro_rs_utterances_get_labels(owner, &mut labels), 0);
        assert_eq!(shiro_rs_labels_length(labels, &mut count), 0);
        assert_eq!(count, expected.labels.len());
        for (index, expected) in expected.labels.iter().enumerate() {
            let mut info = ShiroRsLabelInfo {
                start: 0.0,
                end: 0.0,
            };
            assert_eq!(shiro_rs_labels_get_info(labels, index, &mut info), 0);
            assert_eq!(
                (info.start.to_bits(), info.end.to_bits()),
                (expected.start.to_bits(), expected.end.to_bits())
            );
            let mut name = null_mut();
            assert_eq!(shiro_rs_labels_get_name(labels, index, &mut name), 0);
            assert_eq!(copied(name), expected.name.as_bytes());
            assert_eq!(shiro_rs_bytes_release(&mut name), 0);
        }
        assert_eq!(shiro_rs_phone_map_release(&mut map), 0);
        assert_eq!(shiro_rs_definition_release(&mut definition), 0);
        assert_eq!(shiro_rs_strings_release(&mut phones), 0);
        assert_eq!(shiro_rs_document_release(&mut initial), 0);
        assert_eq!(shiro_rs_document_release(&mut alignment), 0);
        assert_eq!(shiro_rs_iteration_reports_release(&mut reports), 0);
        assert_eq!(shiro_rs_labels_release(&mut labels), 0);
    }
}

unsafe fn reconstruct(owner: *const ShiroRsUtterances) -> *mut ShiroRsUtterances {
    // SAFETY: Independent complete snapshots, immutable borrowed descriptors.
    unsafe {
        let (
            mut map,
            mut definition,
            mut phones,
            mut initial,
            mut alignment,
            mut reports,
            mut labels,
        ) = (
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
        );
        assert_eq!(shiro_rs_utterances_get_phonemap(owner, &mut map), 0);
        assert_eq!(
            shiro_rs_utterances_get_definition(owner, &mut definition),
            0
        );
        assert_eq!(shiro_rs_utterances_get_phones(owner, &mut phones), 0);
        assert_eq!(
            shiro_rs_utterances_get_initial_segmentation(owner, &mut initial),
            0
        );
        assert_eq!(shiro_rs_utterances_get_alignment(owner, &mut alignment), 0);
        assert_eq!(shiro_rs_utterances_get_iterations(owner, &mut reports), 0);
        assert_eq!(shiro_rs_utterances_get_labels(owner, &mut labels), 0);
        let mut models = [null_mut(); 3];
        for (stage, model) in models.iter_mut().enumerate() {
            let mut present = 0;
            assert_eq!(
                shiro_rs_utterances_has_model(owner, stage as u32, &mut present),
                0
            );
            assert_eq!(
                shiro_rs_utterances_get_model(owner, stage as u32, model),
                if present == 1 { 0 } else { 2 }
            );
        }
        let input = ShiroRsUtterancesInput {
            phonemap: map,
            definition,
            phones,
            initial_segmentation: initial,
            uninitialized_model: models[0],
            initialized_model: models[1],
            model: models[2],
            iterations: reports,
            alignment,
            labels,
        };
        let mut output = null_mut();
        assert_eq!(shiro_rs_utterances_create(&input, &mut output), 0);
        assert_eq!(shiro_rs_phone_map_release(&mut map), 0);
        assert_eq!(shiro_rs_definition_release(&mut definition), 0);
        assert_eq!(shiro_rs_strings_release(&mut phones), 0);
        assert_eq!(shiro_rs_document_release(&mut initial), 0);
        assert_eq!(shiro_rs_document_release(&mut alignment), 0);
        assert_eq!(shiro_rs_iteration_reports_release(&mut reports), 0);
        assert_eq!(shiro_rs_labels_release(&mut labels), 0);
        for model in &mut models {
            assert_eq!(shiro_rs_model_release(model), 0);
        }
        output
    }
}

#[test]
fn fresh_initialized_and_trained_workflows_preserve_all_ten_fields_and_complete_reports() {
    let native = features();
    let options = Options {
        utterances: 2,
        iterations: 2,
        ..Options::default()
    };
    for (mode, model) in [
        (0, None),
        (
            1,
            Some(
                Model::read_from(
                    include_bytes!("../../../tests/fixtures/utterances-c-flat.hsmm").as_slice(),
                )
                .unwrap(),
            ),
        ),
        (
            2,
            Some(
                Model::read_from(
                    include_bytes!("../../../tests/fixtures/utterances-c-trained.hsmm").as_slice(),
                )
                .unwrap(),
            ),
        ),
    ] {
        let source = match model.as_ref() {
            None => ModelSource::Fresh,
            Some(model) if mode == 1 => ModelSource::Initialized(model),
            Some(model) => ModelSource::Trained(model),
        };
        let expected =
            utterances::split_features(&native, "sample.param", options, source).unwrap();
        assert_eq!(
            wire(&expected.model),
            include_bytes!("../../../tests/fixtures/utterances-c-trained.hsmm")
        );
        if mode == 0 {
            assert_eq!(
                wire(expected.uninitialized_model.as_ref().unwrap()),
                include_bytes!("../../../tests/fixtures/utterances-c-uninit.hsmm")
            );
            assert_eq!(
                wire(expected.initialized_model.as_ref().unwrap()),
                include_bytes!("../../../tests/fixtures/utterances-c-flat.hsmm")
            );
            assert_eq!(
                serde_json::to_value(&expected.phonemap).unwrap(),
                serde_json::from_slice::<serde_json::Value>(include_bytes!(
                    "../../../tests/fixtures/utterances-c-phonemap.json"
                ))
                .unwrap()
            );
            let definition: shiro_rs::definition::ModelDefinition = serde_json::from_slice(
                include_bytes!("../../../tests/fixtures/utterances-c-definition.json"),
            )
            .unwrap();
            assert_eq!(
                expected.definition.build().unwrap(),
                definition.build().unwrap()
            );
            for (actual, reference) in [
                (
                    &expected.initial_segmentation,
                    include_bytes!("../../../tests/fixtures/utterances-c-initial.json").as_slice(),
                ),
                (
                    &expected.alignment,
                    include_bytes!("../../../tests/fixtures/utterances-c-aligned.json").as_slice(),
                ),
            ] {
                let reference: shiro_rs::labels::SegmentationDocument =
                    serde_json::from_slice(reference).unwrap();
                assert_eq!(
                    serde_json::to_value(actual).unwrap(),
                    serde_json::to_value(reference).unwrap()
                );
            }
        }
        // SAFETY: Live immutable source/input owners and independent output slots.
        unsafe {
            let mut features = feature_owner(&native);
            let mut filename = owned(b"sample.param");
            let mut model_owner = model
                .as_ref()
                .map_or(null_mut(), |value| model_owner(value));
            let source = ShiroRsModelSource {
                mode,
                model: if mode == 0 {
                    std::ptr::dangling::<ShiroRsModel>()
                } else {
                    model_owner
                },
            };
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_utterances_split_features(
                    features,
                    filename,
                    &options.into(),
                    &source,
                    &mut output
                ),
                0
            );
            if let Some(model) = &model {
                assert_eq!(model_wire(model_owner), wire(model));
            }
            assert_eq!(shiro_rs_model_release(&mut model_owner), 0);
            assert_eq!(shiro_rs_features_release(&mut features), 0);
            assert_eq!(shiro_rs_bytes_release(&mut filename), 0);
            compare(output, &expected);
            let mut reconstructed = reconstruct(output);
            let mut cloned = null_mut();
            assert_eq!(shiro_rs_utterances_clone(reconstructed, &mut cloned), 0);
            assert_eq!(shiro_rs_utterances_release(&mut reconstructed), 0);
            assert_eq!(shiro_rs_utterances_release(&mut output), 0);
            compare(cloned, &expected);
            assert_eq!(shiro_rs_utterances_release(&mut cloned), 0);
            assert_eq!(shiro_rs_utterances_release(&mut cloned), 0);
        }
    }
}

#[test]
fn zero_iterations_nondefault_options_and_failures_preserve_native_behavior() {
    let native = features();
    for options in [
        Options {
            utterances: 2,
            iterations: 0,
            ..Options::default()
        },
        Options {
            utterances: 1,
            hop_seconds: 0.125,
            minimum_silence_seconds: 0.25,
            minimum_voicing_seconds: 0.375,
            iterations: 1,
        },
    ] {
        let expected = utterances::split_features(
            &native,
            "audio-\u{1f3b5}\0.param",
            options,
            ModelSource::Fresh,
        )
        .unwrap();
        // SAFETY: Live immutable owners and independent initialized output.
        unsafe {
            let mut input = feature_owner(&native);
            let mut filename = owned("audio-\u{1f3b5}\0.param".as_bytes());
            let source = ShiroRsModelSource {
                mode: 0,
                model: null(),
            };
            let mut settings = ShiroRsUtteranceOptions::from(Options::default());
            assert_eq!(shiro_rs_utterance_options_default(&mut settings), 0);
            assert_eq!(settings, Options::default().into());
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_utterances_split_features(
                    input,
                    filename,
                    &options.into(),
                    &source,
                    &mut output
                ),
                0
            );
            compare(output, &expected);
            for settings in [
                Options {
                    utterances: usize::MAX,
                    ..options
                },
                Options {
                    hop_seconds: 0.0,
                    ..options
                },
                Options {
                    minimum_silence_seconds: f64::NAN,
                    ..options
                },
                Options {
                    minimum_voicing_seconds: f64::INFINITY,
                    ..options
                },
                Options {
                    iterations: usize::MAX,
                    ..options
                },
            ] {
                let mut retained = output;
                assert_eq!(
                    shiro_rs_utterances_split_features(
                        input,
                        filename,
                        &settings.into(),
                        &source,
                        &mut retained
                    ),
                    3
                );
                assert_eq!(retained, output);
            }
            let mut retained = output;
            let invalid_source = ShiroRsModelSource {
                mode: 3,
                model: null(),
            };
            assert_eq!(
                shiro_rs_utterances_split_features(
                    input,
                    filename,
                    &options.into(),
                    &invalid_source,
                    &mut retained
                ),
                2
            );
            assert_eq!(retained, output);
            let missing = ShiroRsModelSource {
                mode: 1,
                model: null(),
            };
            assert_eq!(
                shiro_rs_utterances_split_features(
                    input,
                    filename,
                    &options.into(),
                    &missing,
                    &mut retained
                ),
                1
            );
            assert_eq!(retained, output);
            let mut invalid_name = owned(&[0xff]);
            assert_eq!(
                shiro_rs_utterances_split_features(
                    input,
                    invalid_name,
                    &options.into(),
                    &source,
                    &mut retained
                ),
                3
            );
            assert_eq!(retained, output);
            assert_eq!(shiro_rs_bytes_release(&mut invalid_name), 0);
            let mut present = 73;
            assert_eq!(shiro_rs_utterances_has_model(output, 3, &mut present), 2);
            assert_eq!(present, 73);
            assert_eq!(shiro_rs_utterances_create(null(), &mut retained), 1);
            assert_eq!(retained, output);
            assert_eq!(
                shiro_rs_utterances_split_features(
                    input,
                    filename,
                    &options.into(),
                    &source,
                    null_mut()
                ),
                1
            );
            assert_eq!(shiro_rs_features_release(&mut input), 0);
            assert_eq!(shiro_rs_bytes_release(&mut filename), 0);
            assert_eq!(shiro_rs_utterances_release(&mut output), 0);
        }
    }
}

struct Random {
    sequence: shiro_rs::audio::DitherSequence,
    draws: usize,
    fail_after: usize,
}
unsafe extern "C" fn uniform(context: *mut std::ffi::c_void, output: *mut f32) -> u32 {
    // SAFETY: Live exclusive test context and callback-borrowed output storage.
    unsafe {
        let context = &mut *context.cast::<Random>();
        if context.draws == context.fail_after {
            return 7;
        }
        *output = context.sequence.next_uniform();
        context.draws += 1;
        0
    }
}
unsafe fn floats(owner: *const ShiroRsArrayF32) -> Vec<u32> {
    // SAFETY: Live immutable owner and independent writable output buffers.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_array_f32_length(owner, &mut count), 0);
        let mut values = vec![0.0; count];
        assert_eq!(
            shiro_rs_array_f32_copy(owner, 0, values.as_mut_ptr(), count),
            0
        );
        values.into_iter().map(f32::to_bits).collect()
    }
}

#[test]
fn decoded_wave_all_feature_kinds_complete_results_and_rng_match_native() {
    use shiro_rs::{audio::DitherSequence, features::FeatureKind};
    let original = shiro_rs::dsp::wave::read(
        &mut std::io::Cursor::new(include_bytes!(
            "../../../tests/fixtures/utterances-input.wav"
        )),
        i32::MAX as usize,
    )
    .unwrap();
    for (code, kind) in [
        (0, FeatureKind::Mfcc),
        (1, FeatureKind::Mfbe),
        (2, FeatureKind::Plpcc),
    ] {
        let options = Options {
            utterances: 2,
            iterations: 2,
            ..Options::default()
        };
        let mut draws = 0;
        let mut random = DitherSequence::linux_gnu();
        let expected = utterances::split_wave(
            original.clone(),
            "sample.param",
            13,
            kind,
            options,
            ModelSource::Fresh,
            || {
                draws += 1;
                random.next_uniform()
            },
        )
        .unwrap();
        if code == 0 {
            assert_eq!(
                expected
                    .audio
                    .samples
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                include_bytes!("../../../tests/fixtures/utterances-c-audio.bin")
                    .chunks_exact(4)
                    .map(|value| u32::from_le_bytes(value.try_into().unwrap()))
                    .collect::<Vec<_>>()
            );
        }
        // SAFETY: Complete immutable owners and live synchronous callback context.
        unsafe {
            let mut samples = null_mut();
            assert_eq!(
                shiro_rs_array_f32_create(
                    original.samples.as_ptr(),
                    original.samples.len(),
                    &mut samples
                ),
                0
            );
            let mut filename = owned(b"sample.param");
            let input = ShiroRsWaveSplitInput {
                header: ShiroRsWaveInfo {
                    sample_rate: original.sample_rate,
                    bits_per_sample: original.bits_per_sample,
                    channels: original.channels,
                    encoding: 0,
                },
                samples,
                filename,
                dimensions: 13,
                kind: code,
            };
            let source = ShiroRsModelSource {
                mode: 0,
                model: null(),
            };
            let mut random = Random {
                sequence: DitherSequence::linux_gnu(),
                draws: 0,
                fail_after: usize::MAX,
            };
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_utterances_split_wave(
                    &input,
                    &options.into(),
                    &source,
                    Some(uniform),
                    (&mut random as *mut Random).cast(),
                    &mut output
                ),
                0
            );
            assert_eq!(random.draws, draws);
            let (mut audio, mut features, mut utterances) = (null_mut(), null_mut(), null_mut());
            assert_eq!(shiro_rs_segmented_wave_get_audio(output, &mut audio), 0);
            assert_eq!(
                shiro_rs_segmented_wave_get_features(output, &mut features),
                0
            );
            assert_eq!(
                shiro_rs_segmented_wave_get_utterances(output, &mut utterances),
                0
            );
            let mut rebuilt = null_mut();
            assert_eq!(
                shiro_rs_segmented_wave_create(audio, features, utterances, &mut rebuilt),
                0
            );
            let mut cloned = null_mut();
            assert_eq!(shiro_rs_segmented_wave_clone(rebuilt, &mut cloned), 0);
            assert_eq!(shiro_rs_segmented_wave_release(&mut rebuilt), 0);
            assert_eq!(shiro_rs_segmented_wave_release(&mut output), 0);
            let mut rate = 0;
            assert_eq!(shiro_rs_audio_sample_rate(audio, &mut rate), 0);
            assert_eq!(rate, expected.audio.sample_rate);
            let mut values = null_mut();
            assert_eq!(shiro_rs_audio_get_samples(audio, &mut values), 0);
            assert_eq!(
                floats(values),
                expected
                    .audio
                    .samples
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
            assert_eq!(shiro_rs_array_f32_release(&mut values), 0);
            let mut info = ShiroRsFeatureInfo {
                frames: 0,
                columns: 0,
            };
            assert_eq!(shiro_rs_features_get_info(features, &mut info), 0);
            assert_eq!(
                (info.frames, info.columns),
                (expected.features.frames, expected.features.columns)
            );
            assert_eq!(shiro_rs_features_get_values(features, &mut values), 0);
            assert_eq!(
                floats(values),
                expected
                    .features
                    .values
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
            assert_eq!(shiro_rs_array_f32_release(&mut values), 0);
            compare(utterances, &expected.utterances);
            assert_eq!(shiro_rs_audio_release(&mut audio), 0);
            assert_eq!(shiro_rs_features_release(&mut features), 0);
            assert_eq!(shiro_rs_utterances_release(&mut utterances), 0);
            assert_eq!(
                shiro_rs_segmented_wave_get_utterances(cloned, &mut utterances),
                0
            );
            assert_eq!(shiro_rs_segmented_wave_release(&mut cloned), 0);
            compare(utterances, &expected.utterances);
            assert_eq!(shiro_rs_utterances_release(&mut utterances), 0);
            let mut retained = null_mut();
            random.draws = 0;
            assert_eq!(
                shiro_rs_utterances_split_wave(
                    &input,
                    &options.into(),
                    &source,
                    Some(uniform),
                    (&mut random as *mut Random).cast(),
                    null_mut()
                ),
                1
            );
            assert_eq!(random.draws, 0);
            let invalid = ShiroRsWaveSplitInput {
                dimensions: 1,
                ..input
            };
            assert_eq!(
                shiro_rs_utterances_split_wave(
                    &invalid,
                    &options.into(),
                    &source,
                    Some(uniform),
                    (&mut random as *mut Random).cast(),
                    &mut retained
                ),
                3
            );
            assert_eq!(random.draws, 0);
            let invalid = ShiroRsWaveSplitInput { kind: 3, ..input };
            assert_eq!(
                shiro_rs_utterances_split_wave(
                    &invalid,
                    &options.into(),
                    &source,
                    Some(uniform),
                    (&mut random as *mut Random).cast(),
                    &mut retained
                ),
                2
            );
            assert_eq!(random.draws, 0);
            random.fail_after = 3;
            assert_eq!(
                shiro_rs_utterances_split_wave(
                    &input,
                    &options.into(),
                    &source,
                    Some(uniform),
                    (&mut random as *mut Random).cast(),
                    &mut retained
                ),
                3
            );
            assert_eq!(random.draws, 3);
            assert!(retained.is_null());
            assert_eq!(
                shiro_rs_utterances_split_wave(
                    &input,
                    &options.into(),
                    &source,
                    None,
                    null_mut(),
                    &mut retained
                ),
                3
            );
            assert!(retained.is_null());
            assert_eq!(shiro_rs_array_f32_release(&mut samples), 0);
            assert_eq!(shiro_rs_bytes_release(&mut filename), 0);
        }
    }
}

#[test]
fn arbitrary_complete_results_keep_nonfinite_fields_metadata_and_optional_stage_combinations() {
    let native_model = Model::read_from(
        include_bytes!("../../../tests/fixtures/utterances-c-trained.hsmm").as_slice(),
    )
    .unwrap();
    let time_bits = 0x7ff8_1234_5678_9abc;
    let weight_bits = 0x7fc1_2345;
    let sample_bits = [0x8000_0000, 0x7fc5_4321, 0x7f80_0000, 1];
    let samples = sample_bits.map(f32::from_bits);
    // SAFETY: All input descriptors/owners are live immutable; each output is
    // independent initialized storage and each release transfers unique ownership.
    unsafe {
        let mut map_json = owned(br#"{"phone_map":{},"nested":{"data":[null,true]}}"#);
        let mut map = null_mut();
        assert_eq!(shiro_rs_phone_map_read_json(map_json, &mut map), 0);
        let map_expected: serde_json::Value = serde_json::from_slice(&copied(map_json)).unwrap();
        assert_eq!(shiro_rs_bytes_release(&mut map_json), 0);
        let stream = ShiroRsStreamDefinition {
            states: usize::MAX,
            dimensions: 0,
            mixtures: usize::MAX,
            weight: f32::from_bits(weight_bits),
        };
        let constraint = ShiroRsDurationConstraint {
            index: usize::MAX,
            has_minimum: 1,
            minimum: i32::MIN,
            has_maximum: 1,
            maximum: 0,
        };
        let mut definition = null_mut();
        assert_eq!(
            shiro_rs_definition_create(usize::MAX, &stream, 1, &constraint, 1, &mut definition),
            0
        );
        let mut name = owned(b"voice\0.param");
        let mut empty_name = owned(b"");
        let phone_names = [
            empty_name.cast_const(),
            name.cast_const(),
            empty_name.cast_const(),
        ];
        let mut phones = null_mut();
        assert_eq!(
            shiro_rs_strings_create(phone_names.as_ptr(), phone_names.len(), &mut phones),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut empty_name), 0);
        let mut metadata = owned(br#"[null,"voice",42,{"nested":[true]}]"#);
        let mut state_attrs = owned(br#"{"time":"shadow","dur":false}"#);
        let mut jumps = owned(br#"[[0.25,{"name":"voice"}]]"#);
        let mut outputs = null_mut();
        let integers = [0, usize::MAX];
        assert_eq!(
            shiro_rs_array_usize_create(integers.as_ptr(), integers.len(), &mut outputs),
            0
        );
        let state = ShiroRsStateInput {
            time: f64::from_bits(time_bits),
            has_duration: 1,
            duration: usize::MAX,
            outputs,
            jumps,
            metadata,
            attributes: state_attrs,
        };
        let mut states = null_mut();
        assert_eq!(shiro_rs_states_create(&state, 1, &mut states), 0);
        let state_json_fields = [copied(jumps), copied(metadata), copied(state_attrs)]
            .map(|value| serde_json::from_slice::<serde_json::Value>(&value).unwrap());
        assert_eq!(shiro_rs_array_usize_release(&mut outputs), 0);
        assert_eq!(shiro_rs_bytes_release(&mut jumps), 0);
        assert_eq!(shiro_rs_bytes_release(&mut metadata), 0);
        assert_eq!(shiro_rs_bytes_release(&mut state_attrs), 0);
        let mut file_attrs = owned(br#"{"filename":"shadow","states":false}"#);
        let file_expected: serde_json::Value = serde_json::from_slice(&copied(file_attrs)).unwrap();
        let mut file = null_mut();
        assert_eq!(
            shiro_rs_segmented_file_create(name, states, file_attrs, &mut file),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut file_attrs), 0);
        assert_eq!(shiro_rs_states_release(&mut states), 0);
        let mut attrs = owned(br#"{"file_list":"shadow","nested":[null,7]}"#);
        let doc_expected: serde_json::Value = serde_json::from_slice(&copied(attrs)).unwrap();
        let files = [file.cast_const(), file.cast_const()];
        let mut document = null_mut();
        assert_eq!(
            shiro_rs_document_create(files.as_ptr(), files.len(), attrs, &mut document),
            0
        );
        assert_eq!(shiro_rs_segmented_file_release(&mut file), 0);
        assert_eq!(shiro_rs_bytes_release(&mut attrs), 0);
        let mut model = model_owner(&native_model);
        let report_info = ShiroRsIterationInfo {
            iteration: usize::MAX,
            temperature: f32::from_bits(weight_bits),
            mean_log_likelihood: f32::NEG_INFINITY,
        };
        let mut row = null_mut();
        let mut empty_row = null_mut();
        assert_eq!(
            shiro_rs_array_f32_create(samples.as_ptr(), samples.len(), &mut row),
            0
        );
        assert_eq!(shiro_rs_array_f32_create(null(), 0, &mut empty_row), 0);
        let rows = [row.cast_const(), empty_row.cast_const(), row.cast_const()];
        let mut report = null_mut();
        assert_eq!(
            shiro_rs_iteration_report_create(&report_info, rows.as_ptr(), rows.len(), &mut report),
            0
        );
        assert_eq!(shiro_rs_array_f32_release(&mut row), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut empty_row), 0);
        let report_inputs = [report.cast_const(), report.cast_const()];
        let mut reports = null_mut();
        assert_eq!(
            shiro_rs_iteration_reports_create(
                report_inputs.as_ptr(),
                report_inputs.len(),
                &mut reports
            ),
            0
        );
        assert_eq!(shiro_rs_iteration_report_release(&mut report), 0);
        let label_input = ShiroRsLabelInput {
            start: f64::from_bits(time_bits),
            end: f64::NEG_INFINITY,
            name,
        };
        let mut labels = null_mut();
        assert_eq!(shiro_rs_labels_create(&label_input, 1, &mut labels), 0);
        assert_eq!(shiro_rs_bytes_release(&mut name), 0);
        let mut results = Vec::new();
        for (uninitialized, initialized) in
            [(false, false), (true, false), (false, true), (true, true)]
        {
            let input = ShiroRsUtterancesInput {
                phonemap: map,
                definition,
                phones,
                initial_segmentation: document,
                uninitialized_model: if uninitialized { model } else { null() },
                initialized_model: if initialized { model } else { null() },
                model,
                iterations: reports,
                alignment: document,
                labels,
            };
            let mut output = null_mut();
            assert_eq!(shiro_rs_utterances_create(&input, &mut output), 0);
            let mut retained = output;
            let invalid = ShiroRsUtterancesInput {
                model: null(),
                ..input
            };
            assert_eq!(shiro_rs_utterances_create(&invalid, &mut retained), 1);
            assert_eq!(retained, output);
            let mut cloned = null_mut();
            assert_eq!(shiro_rs_utterances_clone(output, &mut cloned), 0);
            assert_eq!(shiro_rs_utterances_release(&mut output), 0);
            results.push((cloned, uninitialized, initialized));
        }
        assert_eq!(shiro_rs_phone_map_release(&mut map), 0);
        assert_eq!(shiro_rs_definition_release(&mut definition), 0);
        assert_eq!(shiro_rs_strings_release(&mut phones), 0);
        assert_eq!(shiro_rs_document_release(&mut document), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert_eq!(shiro_rs_iteration_reports_release(&mut reports), 0);
        assert_eq!(shiro_rs_labels_release(&mut labels), 0);
        for (mut owner, uninitialized, initialized) in results {
            assert_eq!(shiro_rs_utterances_get_phonemap(owner, &mut map), 0);
            assert_eq!(json(map, shiro_rs_phone_map_write_json), map_expected);
            assert_eq!(shiro_rs_phone_map_release(&mut map), 0);
            assert_eq!(
                shiro_rs_utterances_get_definition(owner, &mut definition),
                0
            );
            let mut info = ShiroRsDefinitionInfo {
                duration_states: 0,
                streams: 0,
                duration_constraints: 0,
            };
            assert_eq!(shiro_rs_definition_get_info(definition, &mut info), 0);
            assert_eq!(
                (
                    info.duration_states,
                    info.streams,
                    info.duration_constraints
                ),
                (usize::MAX, 1, 1)
            );
            let mut actual = stream;
            assert_eq!(
                shiro_rs_definition_get_stream(definition, 0, &mut actual),
                0
            );
            assert_eq!(
                (
                    actual.states,
                    actual.dimensions,
                    actual.mixtures,
                    actual.weight.to_bits()
                ),
                (usize::MAX, 0, usize::MAX, weight_bits)
            );
            let mut bound = constraint;
            assert_eq!(
                shiro_rs_definition_get_constraint(definition, 0, &mut bound),
                0
            );
            assert_eq!(bound, constraint);
            assert_eq!(shiro_rs_definition_release(&mut definition), 0);
            assert_eq!(shiro_rs_utterances_get_phones(owner, &mut phones), 0);
            let mut count = 0;
            assert_eq!(shiro_rs_strings_length(phones, &mut count), 0);
            assert_eq!(count, 3);
            for (index, expected) in [b"".as_slice(), b"voice\0.param".as_slice(), b"".as_slice()]
                .iter()
                .enumerate()
            {
                let mut name = null_mut();
                assert_eq!(shiro_rs_strings_get(phones, index, &mut name), 0);
                assert_eq!(copied(name), *expected);
                assert_eq!(shiro_rs_bytes_release(&mut name), 0);
            }
            assert_eq!(shiro_rs_strings_release(&mut phones), 0);
            for getter in [
                shiro_rs_utterances_get_initial_segmentation,
                shiro_rs_utterances_get_alignment,
            ] {
                assert_eq!(getter(owner, &mut document), 0);
                assert_eq!(shiro_rs_document_length(document, &mut count), 0);
                assert_eq!(count, 2);
                assert_eq!(shiro_rs_document_get_attributes(document, &mut attrs), 0);
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&copied(attrs)).unwrap(),
                    doc_expected
                );
                assert_eq!(shiro_rs_bytes_release(&mut attrs), 0);
                for index in 0..2 {
                    assert_eq!(shiro_rs_document_get_file(document, index, &mut file), 0);
                    assert_eq!(shiro_rs_segmented_file_get_filename(file, &mut name), 0);
                    assert_eq!(copied(name), b"voice\0.param");
                    assert_eq!(shiro_rs_bytes_release(&mut name), 0);
                    assert_eq!(shiro_rs_segmented_file_get_attributes(file, &mut attrs), 0);
                    assert_eq!(
                        serde_json::from_slice::<serde_json::Value>(&copied(attrs)).unwrap(),
                        file_expected
                    );
                    assert_eq!(shiro_rs_bytes_release(&mut attrs), 0);
                    assert_eq!(shiro_rs_segmented_file_get_states(file, &mut states), 0);
                    assert_eq!(shiro_rs_segmented_file_release(&mut file), 0);
                    let mut state_info = ShiroRsStateInfo {
                        time: 0.0,
                        has_duration: 0,
                        duration: 0,
                        has_outputs: 0,
                        has_jumps: 0,
                    };
                    assert_eq!(shiro_rs_states_get_info(states, 0, &mut state_info), 0);
                    assert_eq!(
                        (
                            state_info.time.to_bits(),
                            state_info.duration,
                            state_info.has_duration,
                            state_info.has_outputs,
                            state_info.has_jumps
                        ),
                        (time_bits, usize::MAX, 1, 1, 1)
                    );
                    assert_eq!(shiro_rs_states_get_outputs(states, 0, &mut outputs), 0);
                    let mut integers = [0; 2];
                    assert_eq!(
                        shiro_rs_array_usize_copy(outputs, 0, integers.as_mut_ptr(), 2),
                        0
                    );
                    assert_eq!(integers, [0, usize::MAX]);
                    assert_eq!(shiro_rs_array_usize_release(&mut outputs), 0);
                    for (field, expected) in state_json_fields.iter().enumerate() {
                        assert_eq!(
                            shiro_rs_states_get_json_field(states, 0, field as u32, &mut attrs),
                            0
                        );
                        assert_eq!(
                            serde_json::from_slice::<serde_json::Value>(&copied(attrs)).unwrap(),
                            *expected
                        );
                        assert_eq!(shiro_rs_bytes_release(&mut attrs), 0);
                    }
                    assert_eq!(shiro_rs_states_release(&mut states), 0);
                }
                assert_eq!(shiro_rs_document_release(&mut document), 0);
            }
            for (stage, present) in [(0, uninitialized), (1, initialized), (2, true)] {
                let mut has = 99;
                assert_eq!(shiro_rs_utterances_has_model(owner, stage, &mut has), 0);
                assert_eq!(has, u32::from(present));
                assert_eq!(
                    shiro_rs_utterances_get_model(owner, stage, &mut model),
                    if present { 0 } else { 2 }
                );
                if present {
                    assert_eq!(model_wire(model), wire(&native_model));
                    assert_eq!(shiro_rs_model_release(&mut model), 0);
                } else {
                    assert!(model.is_null());
                }
            }
            assert_eq!(shiro_rs_utterances_get_iterations(owner, &mut reports), 0);
            assert_eq!(shiro_rs_iteration_reports_length(reports, &mut count), 0);
            assert_eq!(count, 2);
            for index in 0..2 {
                assert_eq!(
                    shiro_rs_iteration_reports_get(reports, index, &mut report),
                    0
                );
                let mut actual = report_info;
                assert_eq!(shiro_rs_iteration_report_info(report, &mut actual), 0);
                assert_eq!(
                    (
                        actual.iteration,
                        actual.temperature.to_bits(),
                        actual.mean_log_likelihood.to_bits()
                    ),
                    (usize::MAX, weight_bits, f32::NEG_INFINITY.to_bits())
                );
                assert_eq!(shiro_rs_iteration_report_file_count(report, &mut count), 0);
                assert_eq!(count, 3);
                for row_index in 0..3 {
                    assert_eq!(
                        shiro_rs_iteration_report_get_file(report, row_index, &mut row),
                        0
                    );
                    assert_eq!(
                        floats(row),
                        if row_index == 1 {
                            vec![]
                        } else {
                            sample_bits.to_vec()
                        }
                    );
                    assert_eq!(shiro_rs_array_f32_release(&mut row), 0);
                }
                assert_eq!(shiro_rs_iteration_report_release(&mut report), 0);
            }
            assert_eq!(shiro_rs_iteration_reports_release(&mut reports), 0);
            assert_eq!(shiro_rs_utterances_get_labels(owner, &mut labels), 0);
            assert_eq!(shiro_rs_labels_length(labels, &mut count), 0);
            assert_eq!(count, 1);
            let mut label = ShiroRsLabelInfo {
                start: 0.0,
                end: 0.0,
            };
            assert_eq!(shiro_rs_labels_get_info(labels, 0, &mut label), 0);
            assert_eq!(
                (label.start.to_bits(), label.end.to_bits()),
                (time_bits, f64::NEG_INFINITY.to_bits())
            );
            assert_eq!(shiro_rs_labels_get_name(labels, 0, &mut name), 0);
            assert_eq!(copied(name), b"voice\0.param");
            assert_eq!(shiro_rs_bytes_release(&mut name), 0);
            assert_eq!(shiro_rs_labels_release(&mut labels), 0);
            let mut samples_owner = null_mut();
            assert_eq!(
                shiro_rs_array_f32_create(samples.as_ptr(), samples.len(), &mut samples_owner),
                0
            );
            let mut audio = null_mut();
            assert_eq!(shiro_rs_audio_create(0, samples_owner, &mut audio), 0);
            let mut feature_owner = null_mut();
            assert_eq!(
                shiro_rs_features_create(usize::MAX, 0, samples_owner, &mut feature_owner),
                0
            );
            assert_eq!(shiro_rs_array_f32_release(&mut samples_owner), 0);
            let mut wave = null_mut();
            assert_eq!(
                shiro_rs_segmented_wave_create(audio, feature_owner, owner, &mut wave),
                0
            );
            assert_eq!(shiro_rs_audio_release(&mut audio), 0);
            assert_eq!(shiro_rs_features_release(&mut feature_owner), 0);
            assert_eq!(shiro_rs_utterances_release(&mut owner), 0);
            assert_eq!(shiro_rs_segmented_wave_get_audio(wave, &mut audio), 0);
            assert_eq!(
                shiro_rs_segmented_wave_get_features(wave, &mut feature_owner),
                0
            );
            let mut restored = null_mut();
            assert_eq!(
                shiro_rs_segmented_wave_get_utterances(wave, &mut restored),
                0
            );
            assert_eq!(shiro_rs_segmented_wave_release(&mut wave), 0);
            let mut rate = 99;
            assert_eq!(shiro_rs_audio_sample_rate(audio, &mut rate), 0);
            assert_eq!(rate, 0);
            assert_eq!(shiro_rs_audio_get_samples(audio, &mut samples_owner), 0);
            assert_eq!(floats(samples_owner), sample_bits);
            assert_eq!(shiro_rs_array_f32_release(&mut samples_owner), 0);
            let mut info = ShiroRsFeatureInfo {
                frames: 0,
                columns: 99,
            };
            assert_eq!(shiro_rs_features_get_info(feature_owner, &mut info), 0);
            assert_eq!((info.frames, info.columns), (usize::MAX, 0));
            assert_eq!(
                shiro_rs_features_get_values(feature_owner, &mut samples_owner),
                0
            );
            assert_eq!(floats(samples_owner), sample_bits);
            assert_eq!(shiro_rs_array_f32_release(&mut samples_owner), 0);
            assert_eq!(shiro_rs_audio_release(&mut audio), 0);
            assert_eq!(shiro_rs_features_release(&mut feature_owner), 0);
            assert_eq!(
                shiro_rs_utterances_get_definition(restored, &mut definition),
                0
            );
            assert_eq!(
                shiro_rs_definition_get_stream(definition, 0, &mut actual),
                0
            );
            assert_eq!(actual.weight.to_bits(), weight_bits);
            assert_eq!(shiro_rs_definition_release(&mut definition), 0);
            assert_eq!(shiro_rs_utterances_release(&mut restored), 0);
        }
    }
}

#[test]
fn decoded_header_fields_and_all_existing_model_sources_follow_native_processing() {
    use shiro_rs::dsp::wave::Encoding;
    use shiro_rs::{audio::DitherSequence, features::FeatureKind};
    let original = shiro_rs::dsp::wave::read(
        &mut std::io::Cursor::new(include_bytes!(
            "../../../tests/fixtures/utterances-input.wav"
        )),
        i32::MAX as usize,
    )
    .unwrap();
    for mode in 0..3 {
        let model = Model::read_from(if mode == 1 {
            include_bytes!("../../../tests/fixtures/utterances-c-flat.hsmm").as_slice()
        } else {
            include_bytes!("../../../tests/fixtures/utterances-c-trained.hsmm").as_slice()
        })
        .unwrap();
        for modified in [false, true] {
            let mut wave = original.clone();
            if modified {
                wave.sample_rate /= 2;
                wave.bits_per_sample = 0;
                wave.channels = u16::MAX;
                wave.encoding = Encoding::Float;
            }
            let options = Options {
                utterances: 2,
                iterations: 0,
                ..Options::default()
            };
            let source = match mode {
                0 => ModelSource::Fresh,
                1 => ModelSource::Initialized(&model),
                _ => ModelSource::Trained(&model),
            };
            let mut random = DitherSequence::linux_gnu();
            let mut draws = 0;
            let expected = utterances::split_wave(
                wave.clone(),
                "sample.param",
                13,
                FeatureKind::Mfcc,
                options,
                source,
                || {
                    draws += 1;
                    random.next_uniform()
                },
            )
            .unwrap();
            // SAFETY: Live immutable inputs and model, independent outputs and
            // initialized synchronous callback context with exclusive access.
            unsafe {
                let mut model_owner = model_owner(&model);
                let mut samples = null_mut();
                assert_eq!(
                    shiro_rs_array_f32_create(
                        wave.samples.as_ptr(),
                        wave.samples.len(),
                        &mut samples
                    ),
                    0
                );
                let mut filename = owned(b"sample.param");
                let input = ShiroRsWaveSplitInput {
                    header: ShiroRsWaveInfo {
                        sample_rate: wave.sample_rate,
                        bits_per_sample: wave.bits_per_sample,
                        channels: wave.channels,
                        encoding: u32::from(modified),
                    },
                    samples,
                    filename,
                    dimensions: 13,
                    kind: 0,
                };
                let source = ShiroRsModelSource {
                    mode,
                    model: model_owner,
                };
                let mut random = Random {
                    sequence: DitherSequence::linux_gnu(),
                    draws: 0,
                    fail_after: usize::MAX,
                };
                let mut output = null_mut();
                assert_eq!(
                    shiro_rs_utterances_split_wave(
                        &input,
                        &options.into(),
                        &source,
                        Some(uniform),
                        (&mut random as *mut Random).cast(),
                        &mut output
                    ),
                    0
                );
                assert_eq!(random.draws, draws);
                assert_eq!(model_wire(model_owner), wire(&model));
                assert_eq!(shiro_rs_model_release(&mut model_owner), 0);
                assert_eq!(shiro_rs_array_f32_release(&mut samples), 0);
                assert_eq!(shiro_rs_bytes_release(&mut filename), 0);
                let (mut audio, mut features, mut result) = (null_mut(), null_mut(), null_mut());
                assert_eq!(shiro_rs_segmented_wave_get_audio(output, &mut audio), 0);
                assert_eq!(
                    shiro_rs_segmented_wave_get_features(output, &mut features),
                    0
                );
                assert_eq!(
                    shiro_rs_segmented_wave_get_utterances(output, &mut result),
                    0
                );
                assert_eq!(shiro_rs_segmented_wave_release(&mut output), 0);
                let mut rate = 0;
                assert_eq!(shiro_rs_audio_sample_rate(audio, &mut rate), 0);
                assert_eq!(rate, expected.audio.sample_rate);
                assert_eq!(shiro_rs_audio_get_samples(audio, &mut samples), 0);
                assert_eq!(
                    floats(samples),
                    expected
                        .audio
                        .samples
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>()
                );
                assert_eq!(shiro_rs_array_f32_release(&mut samples), 0);
                let mut info = ShiroRsFeatureInfo {
                    frames: 0,
                    columns: 0,
                };
                assert_eq!(shiro_rs_features_get_info(features, &mut info), 0);
                assert_eq!(
                    (info.frames, info.columns),
                    (expected.features.frames, expected.features.columns)
                );
                assert_eq!(shiro_rs_features_get_values(features, &mut samples), 0);
                assert_eq!(
                    floats(samples),
                    expected
                        .features
                        .values
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>()
                );
                assert_eq!(shiro_rs_array_f32_release(&mut samples), 0);
                compare(result, &expected.utterances);
                assert_eq!(shiro_rs_audio_release(&mut audio), 0);
                assert_eq!(shiro_rs_features_release(&mut features), 0);
                assert_eq!(shiro_rs_utterances_release(&mut result), 0);
            }
        }
    }
}
