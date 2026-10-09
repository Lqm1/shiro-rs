use shiro_rs_capi::*;
#[test]
fn c_abi_csv_has_independent_bytes_and_null_failure_keeps_output() {
    use std::ptr::null_mut;
    // SAFETY: Unique live model/result/byte owners and independent writable slots.
    unsafe {
        let wire = include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm");
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
