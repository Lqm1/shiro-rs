use std::{
    io::Write,
    process::{Command, Stdio},
};
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
fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_shiro-xxcc"))
}
#[test]
fn original_options_file_and_pipe_match_c() {
    let mut reader = Reader(include_bytes!("../../../tests/fixtures/c-xxcc.bin"));
    assert_eq!(reader.bytes(4), b"XCC1");
    assert_eq!(reader.integer(), 72);
    let directory = std::env::temp_dir().join(format!("shiro-xxcc-cli-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("feature input.raw");
    for record in 0..72 {
        let kind = reader.integer();
        let energy = reader.integer();
        let flags = reader.integer();
        let length = reader.integer();
        let hop = reader.scalar();
        let samples = reader.integer();
        let input = reader.bytes(samples * 4).to_vec();
        let frames = reader.integer();
        let columns = reader.integer();
        let expected: Vec<f32> = (0..frames * columns).map(|_| reader.scalar()).collect();
        if ![13, 47, 50].contains(&record) {
            continue;
        }
        let mut args = vec![
            "-f".to_string(),
            ["mfcc", "mfbe", "plpcc"][kind].into(),
            "-m".into(),
            "12".into(),
            "-c".into(),
            "12".into(),
            "-l".into(),
            length.to_string(),
            "-p".into(),
            hop.to_string(),
            "-s".into(),
            "16".into(),
            "-w".into(),
            "400".into(),
            "-W".into(),
            "0.85".into(),
        ];
        if flags & 1 != 0 {
            args.push("-0".into());
        }
        if flags & 6 == 6 {
            args.push("-da".into());
        } else {
            if flags & 2 != 0 {
                args.push("-d".into());
            }
            if flags & 4 != 0 {
                args.push("-a".into());
            }
        }
        if energy != 0 {
            args.extend([
                "-e".into(),
                "-E".into(),
                if energy == 1 { "0".into() } else { "-1".into() },
            ]);
        }
        std::fs::write(&path, &input).unwrap();
        let file = command().args(&args).arg(&path).output().unwrap();
        assert!(
            file.status.success(),
            "{}",
            String::from_utf8_lossy(&file.stderr)
        );
        let mut child = command()
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&input).unwrap();
        let pipe = child.wait_with_output().unwrap();
        assert!(
            pipe.status.success(),
            "{}",
            String::from_utf8_lossy(&pipe.stderr)
        );
        assert_eq!(file.stdout, pipe.stdout);
        assert!(file.stderr.is_empty());
        assert_eq!(file.stdout.len(), expected.len() * 4);
        for (bytes, expected) in file.stdout.chunks_exact(4).zip(expected) {
            let actual = f32::from_le_bytes(bytes.try_into().unwrap());
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else if expected.is_infinite() {
                assert_eq!(actual, expected);
            } else {
                assert!(
                    (f64::from(actual) - f64::from(expected)).abs()
                        / f64::from(expected).abs().max(1.0)
                        < 2e-5
                );
            }
        }
    }
    assert!(reader.0.is_empty());
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
#[test]
fn help_invalid_parameters_and_partial_input() {
    let help = command().arg("-h").output().unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("-0") && help.contains("-W") && help.contains("-E"));
    for args in [
        vec!["-f", "unknown"],
        vec!["-p", "0"],
        vec!["-s", "0"],
        vec!["-m", "0"],
    ] {
        let result = command().args(args).stdin(Stdio::null()).output().unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
    let mut child = command()
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&[1, 2, 3]).unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("partial scalar"));
    assert!(
        command()
            .stdin(Stdio::null())
            .output()
            .unwrap()
            .stdout
            .is_empty()
    );
}
