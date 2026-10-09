use serde_json::json;
use shiro_rs::{hsmm::Model, labels::SegmentationDocument, untying};
use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

unsafe fn owned(data: &[u8]) -> *mut ShiroRsBytes {
    let mut output = null_mut();
    // SAFETY: Readable source and independent output slot.
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
unsafe fn model(data: &[u8]) -> *mut ShiroRsModel {
    // SAFETY: Unique byte owner and independent model output.
    unsafe {
        let mut wire = owned(data);
        let mut output = null_mut();
        assert_eq!(
            shiro_rs_model_read_bytes(wire, 16 * 1024 * 1024, &mut output),
            0
        );
        assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
        output
    }
}
fn document() -> SegmentationDocument {
    serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/align-c-isolated.json"
    ))
    .unwrap()
}

#[test]
fn original_model_document_summary_and_full_weighted_multifile_results() {
    for weighted in [false, true] {
        let input: &[u8] = if weighted {
            include_bytes!("../../../tests/fixtures/untie-c-weighted-input.hsmm")
        } else {
            include_bytes!("../../../tests/fixtures/init-c-aligned.hsmm")
        };
        let native_model = Model::read_from(input).unwrap();
        let mut document = document();
        if weighted {
            document
                .attributes
                .insert("corpus".into(), json!({"retained": true}));
            document.files[0]
                .attributes
                .insert("file_extra".into(), json!([1, "retained"]));
            document.files[0].states[0]
                .attributes
                .insert("state_extra".into(), json!({"nested": null}));
            document.files[0].states[0].jumps = Some(vec![json!({"d":2,"p":0.1})]);
            document.files.push(document.files[0].clone());
        }
        let expected = untying::untie(&native_model, &document).unwrap();
        let mut expected_wire = Vec::new();
        expected.model.write_to(&mut expected_wire).unwrap();
        let mut expected_summary = Vec::new();
        expected.write_summary(&mut expected_summary).unwrap();
        if !weighted {
            assert_eq!(
                expected_wire,
                include_bytes!("../../../tests/fixtures/untie-c.hsmm")
            );
            assert_eq!(
                expected_summary,
                include_bytes!("../../../tests/fixtures/untie-c-summary.txt")
            );
        }
        // SAFETY: Unique owners, immutable borrowed inputs and independent outputs.
        unsafe {
            let mut model = model(input);
            let encoded = owned(&serde_json::to_vec(&document).unwrap());
            let mut result = null_mut();
            assert_eq!(shiro_rs_untie(model, encoded, &mut result), 0);
            let mut clone = null_mut();
            assert_eq!(shiro_rs_untied_model_clone(result, &mut clone), 0);
            assert_eq!(shiro_rs_untied_model_release(&mut result), 0);
            let mut count = 0;
            assert_eq!(shiro_rs_untied_model_length(clone, &mut count), 0);
            assert_eq!(count, expected.assignments.len());
            for (index, value) in expected.assignments.iter().enumerate() {
                let mut actual = ShiroRsAssignment {
                    state: 99,
                    file: 99,
                    segment: 99,
                };
                assert_eq!(
                    shiro_rs_untied_model_get_assignment(clone, index, &mut actual),
                    0
                );
                assert_eq!(
                    actual,
                    ShiroRsAssignment {
                        state: value.state,
                        file: value.file,
                        segment: value.segment
                    }
                );
            }
            let mut snapshot = null_mut();
            let mut doc = null_mut();
            let mut summary = null_mut();
            assert_eq!(shiro_rs_untied_model_get_model(clone, &mut snapshot), 0);
            assert_eq!(shiro_rs_untied_model_get_document(clone, &mut doc), 0);
            assert_eq!(shiro_rs_untied_model_summary_bytes(clone, &mut summary), 0);
            assert_eq!(shiro_rs_untied_model_release(&mut clone), 0);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&bytes(doc)).unwrap(),
                serde_json::to_value(&expected.segmentation).unwrap()
            );
            assert_eq!(bytes(summary), expected_summary);
            let mut wire = null_mut();
            assert_eq!(shiro_rs_model_write_bytes(snapshot, 0, &mut wire), 0);
            assert_eq!(bytes(wire), expected_wire);
            assert_eq!(shiro_rs_bytes_release(&mut wire), 0);
            assert_eq!(shiro_rs_model_write_bytes(model, 0, &mut wire), 0);
            assert_eq!(bytes(wire), input);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&bytes(encoded)).unwrap(),
                serde_json::to_value(document).unwrap()
            );
            for mut owner in [wire, doc, summary, encoded] {
                assert_eq!(shiro_rs_bytes_release(&mut owner), 0);
            }
            assert_eq!(shiro_rs_model_release(&mut snapshot), 0);
            assert_eq!(shiro_rs_model_release(&mut model), 0);
        }
    }
}

#[test]
fn arbitrary_constructor_assignment_fields_and_late_summary_failures() {
    // SAFETY: Unique live owners, readable descriptors and independent output slots.
    unsafe {
        let mut model = model(include_bytes!(
            "../../../tests/fixtures/init-c-aligned.hsmm"
        ));
        let mut input = owned(&serde_json::to_vec(&document()).unwrap());
        let assignments = [ShiroRsAssignment {
            state: usize::MAX,
            file: 0,
            segment: 2,
        }];
        let mut result = null_mut();
        assert_eq!(
            shiro_rs_untied_model_create(model, input, assignments.as_ptr(), 1, &mut result),
            0
        );
        let mut actual = ShiroRsAssignment {
            state: 0,
            file: 0,
            segment: 0,
        };
        assert_eq!(
            shiro_rs_untied_model_get_assignment(result, 0, &mut actual),
            0
        );
        assert_eq!(actual, assignments[0]);
        let mut summary = null_mut();
        assert_eq!(shiro_rs_untied_model_summary_bytes(result, &mut summary), 0);
        assert_eq!(
            bytes(summary),
            format!("{} 0 2 a 2\n", usize::MAX).as_bytes()
        );
        assert_eq!(
            shiro_rs_untied_model_get_assignment(result, 1, &mut actual),
            2
        );
        assert_eq!(actual, assignments[0]);
        let mut retained = result;
        assert_eq!(
            shiro_rs_untied_model_create(model, input, null(), usize::MAX, &mut retained),
            2
        );
        assert_eq!(retained, result);
        let mut malformed = owned(b"{}");
        assert_eq!(shiro_rs_untie(model, malformed, &mut retained), 3);
        assert_eq!(retained, result);
        assert_eq!(shiro_rs_bytes_release(&mut malformed), 0);
        let mut invalid_document = document();
        invalid_document.files[0].states[1].duration = Some(999);
        malformed = owned(&serde_json::to_vec(&invalid_document).unwrap());
        assert_eq!(shiro_rs_untie(model, malformed, &mut retained), 3);
        assert_eq!(retained, result);
        assert_eq!(shiro_rs_bytes_release(&mut malformed), 0);
        let invalid = [
            assignments[0],
            ShiroRsAssignment {
                state: 4,
                file: 99,
                segment: 0,
            },
        ];
        let mut bad = null_mut();
        assert_eq!(
            shiro_rs_untied_model_create(model, input, invalid.as_ptr(), 2, &mut bad),
            0
        );
        let mut retained_summary = summary;
        assert_eq!(
            shiro_rs_untied_model_summary_bytes(bad, &mut retained_summary),
            3
        );
        assert_eq!(retained_summary, summary);
        assert_eq!(shiro_rs_untied_model_release(&mut bad), 0);
        assert_eq!(
            shiro_rs_untied_model_create(model, input, null(), 0, &mut bad),
            0
        );
        let mut empty = null_mut();
        assert_eq!(shiro_rs_untied_model_summary_bytes(bad, &mut empty), 0);
        assert!(bytes(empty).is_empty());
        assert_eq!(shiro_rs_bytes_release(&mut empty), 0);
        assert_eq!(shiro_rs_untied_model_release(&mut bad), 0);
        assert_eq!(shiro_rs_untied_model_release(null_mut()), 1);
        assert_eq!(shiro_rs_untied_model_clone(result, null_mut()), 1);
        assert_eq!(shiro_rs_untied_model_length(result, null_mut()), 1);
        assert_eq!(shiro_rs_untied_model_release(&mut result), 0);
        assert_eq!(shiro_rs_bytes_release(&mut summary), 0);
        assert_eq!(shiro_rs_bytes_release(&mut input), 0);
        assert_eq!(shiro_rs_model_release(&mut model), 0);
    }
}
