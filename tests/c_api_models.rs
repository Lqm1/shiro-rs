#![cfg(feature = "c-api")]
use shiro_rs::{
    c_api::*,
    hsmm::{Model, serial::ModelEncoding},
};
use std::ptr::{null, null_mut};

unsafe fn owned(source: &[u8]) -> *mut ShiroRsBytes {
    let mut result = null_mut();
    // SAFETY: Readable input with an independent initialized output slot.
    assert_eq!(
        unsafe { shiro_rs_bytes_create(source.as_ptr(), source.len(), &mut result) },
        0
    );
    result
}

unsafe fn copied(source: *const ShiroRsBytes) -> Vec<u8> {
    let mut length = 0;
    // SAFETY: Live source and independent initialized output storage.
    unsafe {
        assert_eq!(shiro_rs_bytes_length(source, &mut length), 0);
        let mut values = vec![0; length];
        assert_eq!(
            shiro_rs_bytes_copy(source, 0, values.as_mut_ptr(), length),
            0
        );
        values
    }
}

#[test]
fn definition_preserves_original_c_model_and_independent_lifetimes() {
    // SAFETY: All owners are unique and all participating output slots independent.
    unsafe {
        let mut definition = owned(include_bytes!("fixtures/modeldef.json"));
        let mut model = null_mut();
        let mut clone = null_mut();
        let mut wire = null_mut();
        assert_eq!(shiro_rs_model_from_definition(definition, &mut model), 0);
        assert_eq!(shiro_rs_bytes_release(&mut definition), 0);
        assert_eq!(shiro_rs_model_clone(model, &mut clone), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert!(model.is_null());
        assert_eq!(shiro_rs_model_release(&mut model), 0);
        assert_eq!(shiro_rs_model_write_bytes(clone, 0, &mut wire), 0);
        assert_eq!(shiro_rs_model_release(&mut clone), 0);
        assert_eq!(copied(wire), include_bytes!("fixtures/empty-c.hsmm"));
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
    }
}

#[test]
fn original_models_preserve_all_serialized_fields_in_both_schemas() {
    let fixtures: &[&[u8]] = &[
        include_bytes!("fixtures/empty-c.hsmm"),
        include_bytes!("fixtures/cmu-arctic-all-speakers.hsmm"),
        include_bytes!("fixtures/init-c-multi.hsmm"),
        include_bytes!("fixtures/untie-c-weighted-input.hsmm"),
        include_bytes!("fixtures/rest-c-isolated-daem.hsmm"),
        include_bytes!("fixtures/utterances-c-trained.hsmm"),
    ];
    for original in fixtures {
        let native = Model::read_from(*original).unwrap();
        // SAFETY: Live owners and independent initialized output slots and buffers.
        unsafe {
            let mut source = owned(original);
            let mut model = null_mut();
            assert_eq!(
                shiro_rs_model_read_bytes(source, 16 * 1024 * 1024, &mut model),
                0
            );
            assert_eq!(shiro_rs_bytes_release(&mut source), 0);
            for (code, encoding) in [
                (0, ModelEncoding::WithVarianceFloors),
                (1, ModelEncoding::WithoutVarianceFloors),
            ] {
                let mut expected = Vec::new();
                let success = native.write_with_encoding(&mut expected, encoding).is_ok();
                let mut wire = null_mut();
                assert_eq!(
                    shiro_rs_model_write_bytes(model, code, &mut wire),
                    if success { 0 } else { 3 }
                );
                if success {
                    assert_eq!(copied(wire), expected);
                    let mut reread = null_mut();
                    assert_eq!(
                        shiro_rs_model_read_bytes(wire, 16 * 1024 * 1024, &mut reread),
                        0
                    );
                    let mut rewritten = null_mut();
                    assert_eq!(shiro_rs_model_write_bytes(reread, code, &mut rewritten), 0);
                    assert_eq!(copied(rewritten), expected);
                    assert_eq!(shiro_rs_bytes_release(&mut rewritten), 0);
                    assert_eq!(shiro_rs_model_release(&mut reread), 0);
                    assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
                } else {
                    assert!(wire.is_null());
                }
            }
            assert_eq!(shiro_rs_model_release(&mut model), 0);
        }
    }
}

#[test]
fn model_errors_retain_owner_slots_and_reject_truncated_or_trailing_data() {
    // SAFETY: Live uniquely owned values and separate initialized sentinel slots.
    unsafe {
        let mut definition = owned(include_bytes!("fixtures/modeldef.json"));
        let mut model = null_mut();
        assert_eq!(shiro_rs_model_from_definition(definition, &mut model), 0);
        assert_eq!(shiro_rs_bytes_release(&mut definition), 0);
        for invalid in [
            b"{}".as_slice(),
            b"{",
            b"null",
            b"{} {}",
            b"\xff",
            b"{\"ndurstate\":1,\"streamdef\":[]}",
        ] {
            let mut source = owned(invalid);
            let mut sentinel = model;
            assert_eq!(shiro_rs_model_from_definition(source, &mut sentinel), 3);
            assert_eq!(sentinel, model);
            assert_eq!(shiro_rs_bytes_release(&mut source), 0);
        }
        let original = include_bytes!("fixtures/empty-c.hsmm");
        for end in 0..original.len() {
            let mut source = owned(&original[..end]);
            let mut sentinel = model;
            assert_eq!(
                shiro_rs_model_read_bytes(source, usize::MAX, &mut sentinel),
                3
            );
            assert_eq!(sentinel, model);
            assert_eq!(shiro_rs_bytes_release(&mut source), 0);
        }
        let mut trailing = original.to_vec();
        trailing.push(0);
        let mut source = owned(&trailing);
        let mut sentinel = model;
        assert_eq!(
            shiro_rs_model_read_bytes(source, usize::MAX, &mut sentinel),
            3
        );
        assert_eq!(sentinel, model);
        assert_eq!(shiro_rs_bytes_release(&mut source), 0);
        source = owned(original);
        assert_eq!(shiro_rs_model_read_bytes(source, 0, &mut sentinel), 3);
        assert_eq!(sentinel, model);
        assert_eq!(shiro_rs_model_read_bytes(null(), 0, &mut sentinel), 1);
        assert_eq!(shiro_rs_model_from_definition(null(), &mut sentinel), 1);
        assert_eq!(shiro_rs_model_clone(null(), &mut sentinel), 1);
        assert_eq!(shiro_rs_model_clone(model, null_mut()), 1);
        assert_eq!(shiro_rs_model_read_bytes(source, usize::MAX, null_mut()), 1);
        let mut wire = source;
        assert_eq!(shiro_rs_model_write_bytes(model, 2, &mut wire), 2);
        assert_eq!(wire, source);
        assert_eq!(shiro_rs_model_write_bytes(null(), 0, &mut wire), 1);
        assert_eq!(shiro_rs_model_write_bytes(model, 0, null_mut()), 1);
        assert_eq!(shiro_rs_model_release(null_mut()), 1);
        assert_eq!(shiro_rs_bytes_release(&mut source), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}
