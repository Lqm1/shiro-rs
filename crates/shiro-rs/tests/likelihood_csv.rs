use shiro_rs::{
    hsmm::Model,
    training::{IterationReport, TrainingResult},
};
use std::io::{self, Write};

fn result() -> TrainingResult {
    TrainingResult {
        model: Model::read_from(
            include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm").as_slice(),
        )
        .unwrap(),
        iterations: vec![
            IterationReport {
                iteration: 0,
                temperature: 1.0,
                mean_log_likelihood: 0.0,
                file_likelihoods: vec![
                    vec![-0.0, 1.25, -3.5, f32::INFINITY, f32::NEG_INFINITY, f32::NAN],
                    vec![],
                ],
            },
            IterationReport {
                iteration: 1,
                temperature: 1.0,
                mean_log_likelihood: 0.0,
                file_likelihoods: vec![vec![0.125]],
            },
        ],
    }
}
#[test]
fn exact_format_order_empty_rows_and_partial_io_are_preserved() {
    struct Writer {
        bytes: Vec<u8>,
        interrupted: bool,
        limit: usize,
    }
    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            if self.bytes.len() == self.limit {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            let count = bytes.len().min(3).min(self.limit - self.bytes.len());
            self.bytes.extend_from_slice(&bytes[..count]);
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("CSV must not implicitly flush")
        }
    }
    let result = result();
    let expected = b"-0.000000,1.250000,-3.500000,inf,-inf,NaN\n\n0.125000\n";
    let mut writer = Writer {
        bytes: Vec::new(),
        interrupted: false,
        limit: usize::MAX,
    };
    result.write_likelihood_csv(&mut writer).unwrap();
    assert_eq!(writer.bytes, expected);
    let mut writer = Writer {
        bytes: Vec::new(),
        interrupted: false,
        limit: 7,
    };
    assert_eq!(
        result.write_likelihood_csv(&mut writer).unwrap_err().kind(),
        io::ErrorKind::BrokenPipe
    );
    assert_eq!(writer.bytes, &expected[..7]);
    let mut empty = result.clone();
    empty.iterations.clear();
    let mut bytes = Vec::new();
    empty.write_likelihood_csv(&mut bytes).unwrap();
    assert!(bytes.is_empty());
}
