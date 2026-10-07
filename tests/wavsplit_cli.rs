mod common;
use common::Directory;
use liblrhsmm_rs::Model;
use shiro_rs::{dataset, labels::SegmentationDocument};
use std::{fs, process::Command};

#[test]
fn bare_paths_spaces_all_features_and_loaded_models_work() {
    let directory = Directory::new("wavsplit spaces");
    for kind in ["mfcc", "mfbe", "plpcc"] {
        let filename = format!("{kind} sample.wav");
        fs::write(
            directory.path().join(&filename),
            include_bytes!("fixtures/utterances-input.wav"),
        )
        .unwrap();
        let invoke = |flags: &[&str]| {
            let output = Command::new(env!("CARGO_BIN_EXE_shiro-wavsplit"))
                .current_dir(directory.path())
                .arg(&filename)
                .args(["-n", "2", "-N", "2", "-f", kind])
                .args(flags)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stdout.is_empty());
        };
        invoke(&[]);
        let stem = format!("{kind} sample");
        for suffix in [
            "raw",
            "param",
            "index",
            "phonemap",
            "modeldef",
            "init.segm",
            "aligned.segm",
            "uninit.hsmm",
            "flat.hsmm",
            "trained.hsmm",
            "txt",
        ] {
            assert!(directory.path().join(format!("{stem}.{suffix}")).is_file());
        }
        assert_eq!(
            fs::read_to_string(directory.path().join(format!("{stem}.index"))).unwrap(),
            format!("{stem},sil utt sil utt sil")
        );
        let trained = format!("{stem}.trained.hsmm");
        let bytes = fs::read(directory.path().join(&trained)).unwrap();
        let model = Model::read_from(bytes.as_slice()).unwrap();
        let mut document: SegmentationDocument = serde_json::from_slice(
            &fs::read(directory.path().join(format!("{stem}.aligned.segm"))).unwrap(),
        )
        .unwrap();
        document.files[0].filename = directory
            .path()
            .join(format!("{stem}.param"))
            .to_str()
            .unwrap()
            .to_owned();
        dataset::load(&document, &model, i32::MAX as usize).unwrap();
        let labels = fs::read(directory.path().join(format!("{stem}.txt"))).unwrap();
        // -l takes precedence even if -i points to a nonexistent file.
        invoke(&["-l", &trained, "-i", "missing.hsmm"]);
        assert_eq!(fs::read(directory.path().join(&trained)).unwrap(), bytes);
        assert_eq!(
            fs::read(directory.path().join(format!("{stem}.txt"))).unwrap(),
            labels
        );
        let flat = format!("{stem}.flat.hsmm");
        let flat_bytes = fs::read(directory.path().join(&flat)).unwrap();
        invoke(&["-i", &flat]);
        assert_eq!(fs::read(directory.path().join(&flat)).unwrap(), flat_bytes);
        assert_eq!(fs::read(directory.path().join(&trained)).unwrap(), bytes);
    }
}

#[test]
fn errors_preserve_inputs_and_create_no_intermediate_files() {
    let directory = Directory::new("wavsplit errors");
    let wave = include_bytes!("fixtures/utterances-input.wav");
    for (filename, flags) in [
        ("alias.raw", vec!["-n", "2", "-N", "0"]),
        ("invalid.wav", vec!["-n", "2", "-d", "1"]),
        ("hop.wav", vec!["-n", "2", "-t", "0"]),
        ("missing-model.wav", vec!["-n", "2", "-l", "missing.hsmm"]),
    ] {
        fs::write(directory.path().join(filename), wave).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_shiro-wavsplit"))
            .current_dir(directory.path())
            .arg(filename)
            .args(flags)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(fs::read(directory.path().join(filename)).unwrap(), wave);
        assert!(
            !directory
                .path()
                .join(filename.replace(".wav", ".param").replace(".raw", ".param"))
                .exists()
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_shiro-wavsplit"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
}
