mod common;
use common::Directory;
use std::{fs, process::Command};
#[test]
fn batch_cli_processes_all_rows_and_presets() {
    let directory = Directory::new("fextr-cli");
    let input_index = directory.path().join("index.csv");
    fs::write(&input_index, b"clip one,aa bb\r\n\r\nclip.two,cc\r\n").unwrap();
    for file in ["clip one", "clip.two"] {
        fs::write(
            directory.path().join(format!("{file}.source.wav")),
            include_bytes!("fixtures/c-audio-input.wav"),
        )
        .unwrap();
    }
    for (preset, expected) in [
        (
            "extractor-xxcc-mfcc12-da-16k",
            include_bytes!("fixtures/c-fextr-mfcc12-da.bin").as_slice(),
        ),
        (
            "extractor-xxcc-mfcc12-dae-16k",
            include_bytes!("fixtures/c-fextr-mfcc12-dae.bin").as_slice(),
        ),
        (
            "extractor-xxcc-plpcc12-da-16k",
            include_bytes!("fixtures/c-fextr-plpcc12-da.bin").as_slice(),
        ),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_shiro-fextr"))
            .arg(&input_index)
            .arg("-d")
            .arg(directory.path())
            .args(["-e", ".source.wav", "-r", "0", "-x", preset])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&result.stdout).lines().count(), 2);
        for file in ["clip one", "clip.two"] {
            assert_eq!(
                fs::read(directory.path().join(format!("{file}.raw"))).unwrap(),
                include_bytes!("fixtures/c-audio-input.plain.raw")
            );
            let actual = fs::read(directory.path().join(format!("{file}.param"))).unwrap();
            assert_eq!(actual.len(), expected.len());
            for (a, e) in actual.chunks_exact(4).zip(expected.chunks_exact(4)) {
                let a = f32::from_le_bytes(a.try_into().unwrap()) as f64;
                let e = f32::from_le_bytes(e.try_into().unwrap()) as f64;
                assert!((a - e).abs() / e.abs().max(1.0) < 2e-5);
            }
        }
    }
    let result = Command::new(env!("CARGO_BIN_EXE_shiro-fextr"))
        .arg(&input_index)
        .arg("-d")
        .arg(directory.path())
        .args(["-e", ".source.wav", "-n", "-D", "0.125", "--seed", "1"])
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(
        fs::read(directory.path().join("clip one.raw")).unwrap(),
        fs::read(directory.path().join("clip.two.raw")).unwrap()
    );
}
#[test]
fn cli_rejects_malformed_index_and_reports_missing_inputs() {
    let directory = Directory::new("fextr-errors");
    let input_index = directory.path().join("index.csv");
    fs::write(&input_index, b"clip,aa\n\nbad,bb,cc\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_shiro-fextr"))
        .arg(&input_index)
        .arg("-d")
        .arg(directory.path())
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("line 3"));
    assert!(!directory.path().join("clip.raw").exists());
    fs::write(&input_index, b"missing,aa\n").unwrap();
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_shiro-fextr"))
            .arg(&input_index)
            .arg("-d")
            .arg(directory.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        Command::new(env!("CARGO_BIN_EXE_shiro-fextr"))
            .arg("-h")
            .output()
            .unwrap()
            .status
            .success()
    );
}
