#![cfg(feature = "c-api")]
mod common;
use common::Directory;
use shiro_rs::{
    batch::{self, Extractor, Options, Preset},
    c_api::*,
    index,
};
use std::{
    fs,
    path::Path,
    ptr::{null, null_mut},
};

unsafe extern "C" fn uniform(context: *mut std::ffi::c_void, output: *mut f32) -> u32 {
    // SAFETY: Unique live draw counter and independent library draw slot.
    unsafe {
        *context.cast::<usize>() += 1;
        output.write(0.5);
    }
    0
}
unsafe extern "C" fn failed_uniform(context: *mut std::ffi::c_void, _output: *mut f32) -> u32 {
    // SAFETY: Unique live draw counter; failed callback leaves draw untouched.
    unsafe {
        *context.cast::<usize>() += 1;
    }
    7
}

fn native(path: &Path) -> Vec<u8> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        path.as_os_str()
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect()
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    }
}
unsafe fn owned(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable bytes and independent output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn bytes(owner: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live immutable owner and independent output storage.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_bytes_length(owner, &mut count), 0);
        let mut values = vec![0; count];
        assert_eq!(shiro_rs_bytes_copy(owner, 0, values.as_mut_ptr(), count), 0);
        values
    }
}
unsafe fn path_bytes(data: &[u8]) -> *mut ShiroRsPath {
    // SAFETY: Temporary unique bytes owner and independent path output.
    unsafe {
        let mut wire = owned(data);
        let mut output = null_mut();
        assert_eq!(shiro_rs_path_from_native_bytes(wire, &mut output), 0);
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
        output
    }
}
unsafe fn path(path: &Path) -> *mut ShiroRsPath {
    // SAFETY: Complete native bytes and independent path ownership.
    unsafe { path_bytes(&native(path)) }
}
unsafe fn path_wire(owner: *const ShiroRsPath) -> Vec<u8> {
    // SAFETY: Live immutable path and independent byte snapshot.
    unsafe {
        let mut wire = null_mut();
        assert_eq!(shiro_rs_path_native_bytes(owner, &mut wire), 0);
        let values = bytes(wire);
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
        values
    }
}
unsafe fn output_fields(output: *const ShiroRsBatchOutputs, expected: &batch::Outputs) {
    // SAFETY: Live immutable result and independent path/scalar snapshots.
    unsafe {
        let mut presence = 99;
        assert_eq!(shiro_rs_batch_outputs_has_mfcc(output, &mut presence), 0);
        assert_eq!(presence, u32::from(expected.mfcc.is_some()));
        for (i, value) in [
            Some(&expected.raw),
            Some(&expected.parameters),
            expected.mfcc.as_ref(),
        ]
        .into_iter()
        .enumerate()
        {
            let mut snapshot = null_mut();
            assert_eq!(
                shiro_rs_batch_outputs_get_path(output, i, &mut snapshot),
                if value.is_some() { 0 } else { 2 }
            );
            if let Some(value) = value {
                assert_eq!(path_wire(snapshot), native(value));
                assert_eq!(shiro_rs_path_release(&mut snapshot), 0);
            }
        }
    }
}
#[test]
fn all_presets_settings_and_original_raw_feature_outputs() {
    let directory = Directory::new("c-api-batch-presets");
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
        ),
        (
            Preset::Mfcc12Dae16k,
            include_bytes!("fixtures/c-fextr-mfcc12-dae.bin").as_slice(),
        ),
        (
            Preset::Plpcc12Da16k,
            include_bytes!("fixtures/c-fextr-plpcc12-da.bin").as_slice(),
        ),
    ];
    // SAFETY: Unique live owners and independent outputs; no dither draw needed.
    unsafe {
        let mut stem_owner = path(&stem);
        let mut options = null_mut();
        assert_eq!(shiro_rs_batch_options_default(&mut options), 0);
        for (code, (preset, original)) in cases.into_iter().enumerate() {
            let mut settings = ShiroRsFeatureOptions::from(preset.feature_options());
            assert_eq!(
                shiro_rs_batch_preset_feature_options(code as u32, &mut settings),
                0
            );
            assert_eq!(settings, preset.feature_options().into());
            let mut extractor = null_mut();
            assert_eq!(shiro_rs_extractor_native(code as u32, &mut extractor), 0);
            let mut actual_code = 99;
            assert_eq!(
                shiro_rs_extractor_get_preset(extractor, &mut actual_code),
                0
            );
            assert_eq!(actual_code, code as u32);
            let mut kind = 99;
            assert_eq!(shiro_rs_extractor_kind(extractor, &mut kind), 0);
            assert_eq!(kind, 0);
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_batch_extract_file(
                    stem_owner,
                    options,
                    extractor,
                    None,
                    null_mut(),
                    &mut output
                ),
                0
            );
            let expected = batch::Outputs {
                raw: index::append_suffix(&stem, ".raw"),
                parameters: index::append_suffix(&stem, ".param"),
                mfcc: None,
            };
            output_fields(output, &expected);
            assert_eq!(
                fs::read(&expected.raw).unwrap(),
                include_bytes!("fixtures/c-audio-input.plain.raw")
            );
            let actual = fs::read(&expected.parameters).unwrap();
            assert_eq!(actual.len(), original.len());
            for (a, e) in actual.chunks_exact(4).zip(original.chunks_exact(4)) {
                let a = f32::from_le_bytes(a.try_into().unwrap()) as f64;
                let e = f32::from_le_bytes(e.try_into().unwrap()) as f64;
                assert!((a - e).abs() / e.abs().max(1.0) < 2e-5);
            }
            let mut clone = null_mut();
            assert_eq!(shiro_rs_batch_outputs_clone(output, &mut clone), 0);
            assert_eq!(shiro_rs_batch_outputs_release(&mut output), 0);
            output_fields(clone, &expected);
            assert_eq!(shiro_rs_batch_outputs_release(&mut clone), 0);
            assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
        }
        assert_eq!(shiro_rs_batch_options_release(&mut options), 0);
        assert_eq!(shiro_rs_path_release(&mut stem_owner), 0);
    }
}
#[test]
fn every_option_variant_and_arbitrary_output_field_preserves_source_lifetimes() {
    let units = if cfg!(windows) {
        [0x66u16, 0xd800, 0, 0xdc00]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect()
    } else {
        vec![0x66, 0xff, 0, 0x80]
    };
    // SAFETY: Repeated immutable inputs, unique parents and independent snapshots.
    unsafe {
        let mut audio = ShiroRsAudioOptions {
            normalize: 1,
            dither_level: 0.125,
            has_output_sample_rate: 1,
            output_sample_rate: 8000,
            boundary: 1,
            kernel: 1,
        };
        let mut suffix = owned(b".source\0.wav");
        let mut options = null_mut();
        assert_eq!(
            shiro_rs_batch_options_create(&audio, suffix, &mut options),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut suffix), 0);
        let mut clone = null_mut();
        assert_eq!(shiro_rs_batch_options_clone(options, &mut clone), 0);
        assert_eq!(shiro_rs_batch_options_release(&mut options), 0);
        let expected = audio;
        assert_eq!(shiro_rs_batch_options_get_audio(clone, &mut audio), 0);
        assert_eq!(audio, expected);
        assert_eq!(
            shiro_rs_batch_options_get_input_extension(clone, &mut suffix),
            0
        );
        assert_eq!(shiro_rs_batch_options_release(&mut clone), 0);
        assert_eq!(bytes(suffix), b".source\0.wav");
        assert_eq!(shiro_rs_bytes_release(&mut suffix), 0);
        let mut source = path_bytes(&units);
        let mut extractor = null_mut();
        for kind in [1, 2] {
            assert_eq!(
                if kind == 1 {
                    shiro_rs_extractor_sptk(source, source, source, &mut extractor)
                } else {
                    shiro_rs_extractor_lua(source, source, source, &mut extractor)
                },
                0
            );
            let mut cloned = null_mut();
            assert_eq!(shiro_rs_extractor_clone(extractor, &mut cloned), 0);
            assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
            let mut actual_kind = 99;
            assert_eq!(shiro_rs_extractor_kind(cloned, &mut actual_kind), 0);
            assert_eq!(actual_kind, kind);
            let mut code = 99;
            assert_eq!(shiro_rs_extractor_get_preset(cloned, &mut code), 2);
            assert_eq!(code, 99);
            let mut snapshots = [null_mut(); 3];
            for (i, snapshot) in snapshots.iter_mut().enumerate() {
                assert_eq!(shiro_rs_extractor_get_path(cloned, i, snapshot), 0);
            }
            assert_eq!(shiro_rs_extractor_release(&mut cloned), 0);
            for snapshot in &mut snapshots {
                assert_eq!(path_wire(*snapshot), units);
                assert_eq!(shiro_rs_path_release(snapshot), 0);
            }
        }
        assert_eq!(shiro_rs_extractor_sptk_default(&mut extractor), 0);
        for (i, text) in ["frame", "mfcc", "delta"].into_iter().enumerate() {
            let mut snapshot = null_mut();
            assert_eq!(shiro_rs_extractor_get_path(extractor, i, &mut snapshot), 0);
            assert_eq!(path_wire(snapshot), native(Path::new(text)));
            assert_eq!(shiro_rs_path_release(&mut snapshot), 0);
        }
        assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
        let mut empty = path(Path::new(""));
        for present in [false, true] {
            let mut output = null_mut();
            assert_eq!(
                shiro_rs_batch_outputs_create(
                    source,
                    source,
                    if present { empty } else { null() },
                    &mut output
                ),
                0
            );
            let mut cloned = null_mut();
            assert_eq!(shiro_rs_batch_outputs_clone(output, &mut cloned), 0);
            assert_eq!(shiro_rs_batch_outputs_release(&mut output), 0);
            let mut presence = 99;
            assert_eq!(shiro_rs_batch_outputs_has_mfcc(cloned, &mut presence), 0);
            assert_eq!(presence, u32::from(present));
            let mut snapshots = [null_mut(); 3];
            for (i, snapshot) in snapshots.iter_mut().enumerate() {
                assert_eq!(
                    shiro_rs_batch_outputs_get_path(cloned, i, snapshot),
                    if i != 2 || present { 0 } else { 2 }
                );
            }
            assert_eq!(shiro_rs_batch_outputs_release(&mut cloned), 0);
            for (i, snapshot) in snapshots.iter_mut().enumerate() {
                if !snapshot.is_null() {
                    assert_eq!(
                        path_wire(*snapshot),
                        if i == 2 { Vec::new() } else { units.clone() }
                    );
                    assert_eq!(shiro_rs_path_release(snapshot), 0);
                }
            }
        }
        assert_eq!(shiro_rs_path_release(&mut source), 0);
        assert_eq!(shiro_rs_path_release(&mut empty), 0);
    }
}
#[test]
fn invalid_construction_and_getters_keep_complete_output_owners() {
    // SAFETY: Live sentinels remain unchanged on deliberately rejected inputs.
    unsafe {
        let mut options = null_mut();
        assert_eq!(shiro_rs_batch_options_default(&mut options), 0);
        let original_options = options;
        let mut suffix = owned(&[0xff]);
        let mut audio: ShiroRsAudioOptions = Options::default().audio.into();
        assert_eq!(
            shiro_rs_batch_options_create(&audio, suffix, &mut options),
            3
        );
        assert_eq!(options, original_options);
        assert_eq!(shiro_rs_bytes_release(&mut suffix), 0);
        suffix = owned(b"");
        audio.normalize = 2;
        assert_eq!(
            shiro_rs_batch_options_create(&audio, suffix, &mut options),
            2
        );
        assert_eq!(options, original_options);
        assert_eq!(
            shiro_rs_batch_options_create(null(), suffix, &mut options),
            1
        );
        let mut extractor = null_mut();
        assert_eq!(shiro_rs_extractor_native(0, &mut extractor), 0);
        let original_extractor = extractor;
        assert_eq!(shiro_rs_extractor_native(3, &mut extractor), 2);
        assert_eq!(extractor, original_extractor);
        let mut settings = Preset::default().feature_options().into();
        let before = settings;
        assert_eq!(shiro_rs_batch_preset_feature_options(3, &mut settings), 2);
        assert_eq!(settings, before);
        let mut path_owner = path(Path::new("sentinel"));
        let original_path = path_owner;
        assert_eq!(
            shiro_rs_extractor_get_path(extractor, 0, &mut path_owner),
            2
        );
        assert_eq!(path_owner, original_path);
        assert_eq!(
            shiro_rs_extractor_sptk(path_owner, null(), path_owner, &mut extractor),
            1
        );
        assert_eq!(extractor, original_extractor);
        assert_eq!(
            shiro_rs_extractor_lua(path_owner, path_owner, null(), &mut extractor),
            1
        );
        assert_eq!(extractor, original_extractor);
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_batch_outputs_create(path_owner, path_owner, null(), &mut output),
            0
        );
        let original_output = output;
        assert_eq!(
            shiro_rs_batch_outputs_create(null(), path_owner, null(), &mut output),
            1
        );
        assert_eq!(output, original_output);
        for i in [2, usize::MAX] {
            assert_eq!(
                shiro_rs_batch_outputs_get_path(output, i, &mut path_owner),
                2
            );
            assert_eq!(path_owner, original_path);
        }
        assert_eq!(shiro_rs_batch_options_clone(options, null_mut()), 1);
        assert_eq!(shiro_rs_extractor_clone(extractor, null_mut()), 1);
        assert_eq!(shiro_rs_batch_outputs_clone(output, null_mut()), 1);
        assert_eq!(shiro_rs_batch_options_release(null_mut()), 1);
        assert_eq!(shiro_rs_extractor_release(null_mut()), 1);
        assert_eq!(shiro_rs_batch_outputs_release(null_mut()), 1);
        assert_eq!(shiro_rs_batch_outputs_release(&mut output), 0);
        assert_eq!(shiro_rs_batch_outputs_release(&mut output), 0);
        assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
        assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
        assert_eq!(shiro_rs_batch_options_release(&mut options), 0);
        assert_eq!(shiro_rs_batch_options_release(&mut options), 0);
        assert_eq!(shiro_rs_bytes_release(&mut suffix), 0);
        assert_eq!(shiro_rs_path_release(&mut path_owner), 0);
    }
}
#[test]
fn native_options_failures_and_side_effect_order_match_direct_extraction() {
    let directory = Directory::new("c-api-batch-options");
    let stem = directory.path().join("clip");
    fs::write(
        index::append_suffix(&stem, ".source.wav"),
        include_bytes!("fixtures/c-audio-input.wav"),
    )
    .unwrap();
    let options = Options {
        input_extension: ".source.wav".into(),
        audio: shiro_rs::audio::AudioOptions {
            normalize: true,
            output_sample_rate: Some(8000),
            boundary: ciglet_rs::resampling::BoundaryPolicy::LegacySkipFirst,
            kernel: ciglet_rs::resampling::KernelPolicy::Legacy,
            ..Default::default()
        },
    };
    let reference = batch::extract_file(
        &stem,
        &options,
        &Extractor::Native(Preset::default()),
        || 0.5,
    )
    .unwrap();
    let expected_raw = fs::read(&reference.raw).unwrap();
    let expected_param = fs::read(&reference.parameters).unwrap();
    // SAFETY: Independent complete options/paths, native extractor and output.
    unsafe {
        let mut stem_owner = path(&stem);
        let mut extension = owned(options.input_extension.as_bytes());
        let audio = options.audio.into();
        let mut settings = null_mut();
        let mut extractor = null_mut();
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_batch_options_create(&audio, extension, &mut settings),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut extension), 0);
        assert_eq!(shiro_rs_extractor_native(0, &mut extractor), 0);
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                settings,
                extractor,
                None,
                null_mut(),
                &mut output
            ),
            0
        );
        output_fields(output, &reference);
        assert_eq!(fs::read(&reference.raw).unwrap(), expected_raw);
        assert_eq!(fs::read(&reference.parameters).unwrap(), expected_param);
        let original_output = output;
        let mut missing = path(&directory.path().join("missing"));
        assert_eq!(
            shiro_rs_batch_extract_file(
                missing,
                settings,
                extractor,
                None,
                null_mut(),
                &mut output
            ),
            3
        );
        assert_eq!(output, original_output);
        assert!(!directory.path().join("missing.raw").exists());
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                settings,
                extractor,
                None,
                null_mut(),
                null_mut()
            ),
            1
        );
        assert_eq!(fs::read(&reference.raw).unwrap(), expected_raw);
        assert_eq!(shiro_rs_batch_options_release(&mut settings), 0);
        let dither = ShiroRsAudioOptions {
            dither_level: 0.2,
            ..audio
        };
        extension = owned(b".source.wav");
        assert_eq!(
            shiro_rs_batch_options_create(&dither, extension, &mut settings),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut extension), 0);
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                settings,
                extractor,
                None,
                null_mut(),
                &mut output
            ),
            3
        );
        assert_eq!(output, original_output);
        assert_eq!(fs::read(&reference.raw).unwrap(), expected_raw);
        let mut native_draws = 0usize;
        let native_options = Options {
            audio: shiro_rs::audio::AudioOptions {
                dither_level: 0.2,
                ..options.audio
            },
            ..options.clone()
        };
        let native_dither = batch::extract_file(
            &stem,
            &native_options,
            &Extractor::Native(Preset::default()),
            || {
                native_draws += 1;
                0.5
            },
        )
        .unwrap();
        let native_raw = fs::read(&native_dither.raw).unwrap();
        let native_parameters = fs::read(&native_dither.parameters).unwrap();
        let mut draws = 0usize;
        let mut dither_result = null_mut();
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                settings,
                extractor,
                Some(uniform),
                std::ptr::from_mut(&mut draws).cast(),
                &mut dither_result
            ),
            0
        );
        assert_eq!(draws, native_draws);
        assert!(draws > 0);
        output_fields(dither_result, &native_dither);
        assert_eq!(fs::read(&native_dither.raw).unwrap(), native_raw);
        assert_eq!(
            fs::read(&native_dither.parameters).unwrap(),
            native_parameters
        );
        assert_eq!(shiro_rs_batch_outputs_release(&mut dither_result), 0);
        draws = 0;
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                settings,
                extractor,
                Some(failed_uniform),
                std::ptr::from_mut(&mut draws).cast(),
                &mut output
            ),
            3
        );
        assert_eq!(draws, 1);
        assert_eq!(output, original_output);
        assert_eq!(fs::read(&native_dither.raw).unwrap(), native_raw);
        assert_eq!(shiro_rs_batch_options_release(&mut settings), 0);
        extension = owned(b".raw");
        assert_eq!(
            shiro_rs_batch_options_create(&audio, extension, &mut settings),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut extension), 0);
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                settings,
                extractor,
                None,
                null_mut(),
                &mut output
            ),
            3
        );
        assert_eq!(output, original_output);
        assert_eq!(fs::read(&reference.raw).unwrap(), expected_raw);
        assert_eq!(shiro_rs_batch_outputs_release(&mut output), 0);
        assert_eq!(shiro_rs_batch_options_release(&mut settings), 0);
        assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
        assert_eq!(shiro_rs_path_release(&mut stem_owner), 0);
        assert_eq!(shiro_rs_path_release(&mut missing), 0);
    }
}
#[test]
#[ignore = "requires SHIRO_TEST_LUA and SHIRO_TEST_SPTK_DIRECTORY protocol fixtures"]
fn all_external_extractor_fields_protocols_and_failure_outputs() {
    let tools = std::path::PathBuf::from(
        std::env::var_os("SHIRO_TEST_SPTK_DIRECTORY").expect("SPTK protocol fixtures required"),
    );
    let lua = std::path::PathBuf::from(
        std::env::var_os("SHIRO_TEST_LUA").expect("Lua interpreter required"),
    );
    let directory = Directory::new("c-api-batch-external");
    let stem = directory.path().join("sample with spaces");
    fs::write(
        index::append_suffix(&stem, ".wav"),
        include_bytes!("fixtures/c-audio-input.wav"),
    )
    .unwrap();
    let script = directory.path().join("extractor.lua");
    fs::write(&script, "return function(try_execute, stem, rawfile, prefix) assert(try_execute == _G.try_execute); assert(#prefix > 0); local f = assert(io.open(rawfile, 'rb')); local bytes = f:read('*a'); f:close(); local o = assert(io.open(stem .. '.param', 'wb')); o:write(bytes); o:close() end\n").unwrap();
    // SAFETY: Full live host paths and independent result ownership.
    unsafe {
        let mut stem_owner = path(&stem);
        let mut options = null_mut();
        assert_eq!(shiro_rs_batch_options_default(&mut options), 0);
        let mut programs = [
            path(&index::append_suffix(
                &tools.join("frame"),
                std::env::consts::EXE_SUFFIX,
            )),
            path(&index::append_suffix(
                &tools.join("mfcc"),
                std::env::consts::EXE_SUFFIX,
            )),
            path(&index::append_suffix(
                &tools.join("delta"),
                std::env::consts::EXE_SUFFIX,
            )),
        ];
        let mut extractor = null_mut();
        assert_eq!(
            shiro_rs_extractor_sptk(programs[0], programs[1], programs[2], &mut extractor),
            0
        );
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                options,
                extractor,
                None,
                null_mut(),
                &mut output
            ),
            0
        );
        let expected = batch::Outputs {
            raw: index::append_suffix(&stem, ".raw"),
            parameters: index::append_suffix(&stem, ".param"),
            mfcc: Some(index::append_suffix(&stem, ".mfcc")),
        };
        output_fields(output, &expected);
        for file in [
            &expected.raw,
            &expected.parameters,
            expected.mfcc.as_ref().unwrap(),
        ] {
            assert_eq!(
                fs::read(file).unwrap(),
                include_bytes!("fixtures/c-audio-input.plain.raw")
            );
        }
        assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
        let mut failing = path(&index::append_suffix(
            &tools.join("mfcc-fail"),
            std::env::consts::EXE_SUFFIX,
        ));
        assert_eq!(
            shiro_rs_extractor_sptk(programs[0], failing, programs[2], &mut extractor),
            0
        );
        let retained = output;
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                options,
                extractor,
                None,
                null_mut(),
                &mut output
            ),
            3
        );
        assert_eq!(output, retained);
        assert_eq!(
            fs::read(&expected.raw).unwrap(),
            include_bytes!("fixtures/c-audio-input.plain.raw")
        );
        assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
        assert_eq!(shiro_rs_path_release(&mut failing), 0);
        for value in &mut programs {
            assert_eq!(shiro_rs_path_release(value), 0);
        }
        let mut paths = [path(&lua), path(&script), path(directory.path())];
        assert_eq!(
            shiro_rs_extractor_lua(paths[0], paths[1], paths[2], &mut extractor),
            0
        );
        assert_eq!(shiro_rs_batch_outputs_release(&mut output), 0);
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                options,
                extractor,
                None,
                null_mut(),
                &mut output
            ),
            0
        );
        output_fields(
            output,
            &batch::Outputs {
                mfcc: None,
                ..expected
            },
        );
        assert_eq!(
            fs::read(index::append_suffix(&stem, ".param")).unwrap(),
            include_bytes!("fixtures/c-audio-input.plain.raw")
        );
        fs::write(&script, "return 42\n").unwrap();
        let retained = output;
        assert_eq!(
            shiro_rs_batch_extract_file(
                stem_owner,
                options,
                extractor,
                None,
                null_mut(),
                &mut output
            ),
            3
        );
        assert_eq!(output, retained);
        assert_eq!(shiro_rs_extractor_release(&mut extractor), 0);
        for value in &mut paths {
            assert_eq!(shiro_rs_path_release(value), 0);
        }
        assert_eq!(shiro_rs_batch_outputs_release(&mut output), 0);
        assert_eq!(shiro_rs_batch_options_release(&mut options), 0);
        assert_eq!(shiro_rs_path_release(&mut stem_owner), 0);
    }
}
