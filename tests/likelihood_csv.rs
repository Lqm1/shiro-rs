use shiro_rs::{
    hsmm::Model,
    training::{IterationReport, TrainingResult},
};
use std::io::{self, Write};

fn result() -> TrainingResult {
    TrainingResult {
        model: Model::read_from(include_bytes!("fixtures/init-c-aligned.hsmm").as_slice()).unwrap(),
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
#[cfg(feature = "c-api")]
#[test]
fn c_abi_csv_has_independent_bytes_and_null_failure_keeps_output() {
    use shiro_rs::c_api::*;
    use std::ptr::null_mut;
    // SAFETY: Unique live model/result/byte owners and independent writable slots.
    unsafe {
        let wire = include_bytes!("fixtures/init-c-aligned.hsmm");
        let mut input = null_mut();
        assert_eq!(
            shiro_rs_bytes_create(wire.as_ptr(), wire.len(), &mut input),
            0
        );
        let mut model = null_mut();
        assert_eq!(shiro_rs_model_read_bytes(input, 1000000, &mut model), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        let mut files = null_mut();
        assert_eq!(
            shiro_rs_training_files_create(std::ptr::null(), 0, &mut files),
            0
        );
        let mut options = std::mem::zeroed();
        assert_eq!(shiro_rs_training_options_default(&mut options), 0);
        options.iterations = 0;
        let mut trained = null_mut();
        assert_eq!(shiro_rs_train(model, files, &options, &mut trained), 0);
        let mut bytes = null_mut();
        assert_eq!(
            shiro_rs_training_result_likelihood_csv_bytes(trained, &mut bytes),
            0
        );
        let original = bytes;
        assert_ne!(
            shiro_rs_training_result_likelihood_csv_bytes(std::ptr::null(), &mut bytes),
            0
        );
        assert_eq!(bytes, original);
        let mut length = usize::MAX;
        assert_eq!(shiro_rs_bytes_length(bytes, &mut length), 0);
        assert_eq!(length, 0);
        assert_eq!(shiro_rs_training_result_release(&mut trained), 0);
        assert_eq!(shiro_rs_bytes_release(&mut bytes), 0);
        assert_eq!(shiro_rs_training_files_release(&mut files), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}
