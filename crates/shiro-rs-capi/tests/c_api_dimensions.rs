use shiro_rs::{dataset, hsmm::Model};
use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

unsafe fn bytes(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable bytes and independent output.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(data.as_ptr(), data.len(), &mut output) },
        0
    );
    output
}
unsafe fn copied(values: *const ShiroRsArrayUsize) -> Vec<usize> {
    // SAFETY: Live owner, independent length and complete output buffer.
    unsafe {
        let mut count = 0;
        assert_eq!(shiro_rs_array_usize_length(values, &mut count), 0);
        let mut output = vec![0; count];
        assert_eq!(
            shiro_rs_array_usize_copy(values, 0, output.as_mut_ptr(), count),
            0
        );
        output
    }
}

#[test]
fn all_original_model_stream_dimensions_and_independent_snapshots_match_native() {
    let fixtures: &[&[u8]] = &[
        include_bytes!("../../../tests/fixtures/empty-c.hsmm"),
        include_bytes!("../../../tests/fixtures/cmu-arctic-all-speakers.hsmm"),
        include_bytes!("../../../tests/fixtures/init-c-multi.hsmm"),
        include_bytes!("../../../tests/fixtures/untie-c-weighted-input.hsmm"),
    ];
    for wire in fixtures {
        let expected = dataset::dimensions(&Model::read_from(*wire).unwrap()).unwrap();
        // SAFETY: Unique live owners and independent output storage throughout.
        unsafe {
            let mut wire = bytes(wire);
            let mut model = null_mut();
            assert_eq!(
                shiro_rs_model_read_bytes(wire, 16 * 1024 * 1024, &mut model),
                0
            );
            assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
            let mut values = null_mut();
            let mut clone = null_mut();
            assert_eq!(shiro_rs_model_dimensions(model, &mut values), 0);
            assert_eq!(shiro_rs_array_usize_clone(values, &mut clone), 0);
            assert_eq!(shiro_rs_array_usize_release(&mut values), 0);
            assert_eq!(shiro_rs_model_dimensions(model, &mut values), 0);
            assert_eq!(shiro_rs_model_release(&mut model), 0);
            assert_eq!(copied(values), expected);
            assert_eq!(copied(clone), expected);
            assert_eq!(shiro_rs_array_usize_release(&mut values), 0);
            assert_eq!(shiro_rs_array_usize_release(&mut clone), 0);
        }
    }
    // SAFETY: Live independently owned definition/model/array output slots.
    unsafe {
        let mut definition = bytes(br#"{"ndurstate":1,"streamdef":[{"nstate":1,"ndim":7},{"nstate":2,"ndim":1},{"nstate":1,"ndim":19}]}"#);
        let mut model = null_mut();
        let mut values = null_mut();
        assert_eq!(shiro_rs_model_from_definition(definition, &mut model), 0);
        assert_eq!(shiro_rs_bytes_release(&mut definition), 0);
        assert_eq!(shiro_rs_model_dimensions(model, &mut values), 0);
        assert_eq!(shiro_rs_model_dimensions(model, null_mut()), 1);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert_eq!(copied(values), [7, 1, 19]);
        assert_eq!(shiro_rs_array_usize_release(&mut values), 0);
    }
}

#[test]
fn arbitrary_target_width_values_empty_ranges_and_failed_outputs_are_preserved() {
    // SAFETY: Unique owners and independent valid buffers except deliberately
    // rejected null/misaligned/oversized descriptors, never dereferenced.
    unsafe {
        let mut input = [0, usize::MAX, usize::MAX - 1, 1];
        let mut values = null_mut();
        assert_eq!(
            shiro_rs_array_usize_create(input.as_ptr(), input.len(), &mut values),
            0
        );
        input.fill(42);
        assert_eq!(copied(values), [0, usize::MAX, usize::MAX - 1, 1]);
        let mut output = [73; 4];
        assert_eq!(
            shiro_rs_array_usize_copy(values, 1, output.as_mut_ptr(), 2),
            0
        );
        assert_eq!(output, [usize::MAX, usize::MAX - 1, 73, 73]);
        output.fill(73);
        for (offset, count) in [(3, 2), (usize::MAX, 1), (5, 0)] {
            assert_eq!(
                shiro_rs_array_usize_copy(values, offset, output.as_mut_ptr(), count),
                2
            );
            assert_eq!(output, [73; 4]);
        }
        assert_eq!(shiro_rs_array_usize_copy(values, 4, null_mut(), 0), 0);
        assert_eq!(shiro_rs_array_usize_copy(values, 0, null_mut(), 1), 1);
        let mut retained = values;
        assert_eq!(shiro_rs_array_usize_create(null(), 1, &mut retained), 1);
        assert_eq!(retained, values);
        assert_eq!(
            shiro_rs_array_usize_create(null(), usize::MAX, &mut retained),
            2
        );
        assert_eq!(retained, values);
        assert_eq!(
            shiro_rs_array_usize_create(std::ptr::without_provenance(1), 1, &mut retained),
            1
        );
        assert_eq!(retained, values);
        assert_eq!(shiro_rs_model_dimensions(null(), &mut retained), 1);
        assert_eq!(retained, values);
        assert_eq!(shiro_rs_array_usize_clone(values, null_mut()), 1);
        assert_eq!(shiro_rs_array_usize_length(values, null_mut()), 1);
        assert_eq!(shiro_rs_array_usize_release(null_mut()), 1);
        let mut clone = null_mut();
        assert_eq!(shiro_rs_array_usize_clone(values, &mut clone), 0);
        assert_eq!(shiro_rs_array_usize_release(&mut values), 0);
        assert_eq!(copied(clone), [0, usize::MAX, usize::MAX - 1, 1]);
        assert_eq!(shiro_rs_array_usize_release(&mut clone), 0);
        assert_eq!(shiro_rs_array_usize_create(null(), 0, &mut values), 0);
        assert!(copied(values).is_empty());
        assert_eq!(shiro_rs_array_usize_release(&mut values), 0);
        assert_eq!(shiro_rs_array_usize_release(&mut values), 0);
    }
}
