use std::{fs, process::Command};
fn run(input: &std::path::Path, flags: &[&str], extension: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_shiro-wav2raw"))
        .args(flags)
        .args(["-e", extension])
        .arg(input)
        .output()
        .unwrap()
}
#[test]
fn original_c_wav_conversion_and_feature_pipeline() {
    let directory = std::env::temp_dir().join(format!("shiro-wav2raw-c-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("input audio.wav");
    fs::write(&input, include_bytes!("fixtures/c-audio-input.wav")).unwrap();
    let cases: &[(&str, &[&str], &[u8])] = &[
        (
            ".plain.raw",
            &[],
            include_bytes!("fixtures/c-audio-input.plain.raw"),
        ),
        (
            ".normalized.raw",
            &["-N"],
            include_bytes!("fixtures/c-audio-input.normalized.raw"),
        ),
        (
            ".up.raw",
            &["-r", "32000", "--legacy-resample"],
            include_bytes!("fixtures/c-audio-input.up.raw"),
        ),
        (
            ".down.raw",
            &["-r", "8000", "--legacy-resample"],
            include_bytes!("fixtures/c-audio-input.down.raw"),
        ),
        (
            ".normalized-down.raw",
            &["-N", "-r", "8000", "--legacy-resample"],
            include_bytes!("fixtures/c-audio-input.normalized-down.raw"),
        ),
    ];
    for (extension, flags, expected) in cases {
        let result = run(&input, flags, extension);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty());
        let actual = fs::read(directory.join(format!("input audio{extension}"))).unwrap();
        assert_eq!(actual.len(), expected.len());
        let mut maximum = 0.0f64;
        for (actual, expected) in actual.chunks_exact(4).zip(expected.chunks_exact(4)) {
            let a = f32::from_le_bytes(actual.try_into().unwrap()) as f64;
            let e = f32::from_le_bytes(expected.try_into().unwrap()) as f64;
            let difference = (a - e).abs() / e.abs().max(1.0);
            maximum = maximum.max(difference);
            assert!(difference < 2e-7, "{extension}: {a} vs {e}");
        }
        eprintln!("WAV conversion {extension}: normalized difference {maximum:e}");
    }
    if cfg!(target_os = "linux") {
        assert!(
            run(&input, &["-d", "0.125"], ".dither.raw")
                .status
                .success()
        );
        assert_eq!(
            fs::read(directory.join("input audio.dither.raw")).unwrap(),
            include_bytes!("fixtures/c-audio-input.dither-linux.raw")
        );
    }
    let raw = directory.join("input audio.plain.raw");
    let result = Command::new(env!("CARGO_BIN_EXE_shiro-xxcc"))
        .args(["-l", "512", "-p", "32", "-s", "16"])
        .arg(raw)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout.len(), 8 * 12 * 4);
    assert!(
        result
            .stdout
            .chunks_exact(4)
            .all(|x| f32::from_le_bytes(x.try_into().unwrap()).is_finite())
    );
    for entry in fs::read_dir(&directory).unwrap() {
        fs::remove_file(entry.unwrap().path()).unwrap();
    }
    fs::remove_dir(directory).unwrap();
}
#[test]
fn paths_errors_silence_and_seeded_dither() {
    let directory =
        std::env::temp_dir().join(format!("shiro-wav2raw-validation-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("single");
    fs::write(&input, include_bytes!("fixtures/c-audio-input.wav")).unwrap();
    assert!(run(&input, &[], ".raw").status.success());
    assert_eq!(
        fs::read(directory.join("single.raw")).unwrap(),
        include_bytes!("fixtures/c-audio-input.plain.raw")
    );
    fs::write(
        directory.join("a"),
        include_bytes!("fixtures/c-audio-input.wav"),
    )
    .unwrap();
    let relative = Command::new(env!("CARGO_BIN_EXE_shiro-wav2raw"))
        .current_dir(&directory)
        .args(["-e", ".relative.raw", "a"])
        .output()
        .unwrap();
    assert!(relative.status.success());
    assert_eq!(
        fs::read(directory.join("a.relative.raw")).unwrap(),
        include_bytes!("fixtures/c-audio-input.plain.raw")
    );
    assert!(!directory.join(".relative.raw").exists());
    for flags in [&["-r", "0"][..], &["-d", "NaN"][..], &["-r", "-1"][..]] {
        let result = run(&input, flags, ".invalid.raw");
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(!directory.join("single.invalid.raw").exists());
    }
    assert!(!run(&input, &[], "").status.success());
    assert_eq!(
        fs::read(&input).unwrap(),
        include_bytes!("fixtures/c-audio-input.wav")
    );
    for extension in [".seed-a.raw", ".seed-b.raw"] {
        assert!(
            run(&input, &["-d", "0.125", "--seed", "42"], extension)
                .status
                .success()
        );
    }
    assert_eq!(
        fs::read(directory.join("single.seed-a.raw")).unwrap(),
        fs::read(directory.join("single.seed-b.raw")).unwrap()
    );
    let mut silence = include_bytes!("fixtures/c-audio-input.wav").to_vec();
    silence[44..].fill(0);
    let silent = directory.join("silence.wav");
    fs::write(&silent, silence).unwrap();
    assert!(run(&silent, &["-N"], ".raw").status.success());
    assert_eq!(
        fs::read(directory.join("silence.raw")).unwrap(),
        vec![0; 257 * 4]
    );
    let broken = directory.join("broken.wav");
    fs::write(&broken, b"RIFF").unwrap();
    assert!(!run(&broken, &[], ".raw").status.success());
    assert!(!directory.join("broken.raw").exists());
    assert!(
        Command::new(env!("CARGO_BIN_EXE_shiro-wav2raw"))
            .arg("-h")
            .output()
            .unwrap()
            .status
            .success()
    );
    for entry in fs::read_dir(&directory).unwrap() {
        fs::remove_file(entry.unwrap().path()).unwrap();
    }
    fs::remove_dir(directory).unwrap();
}
