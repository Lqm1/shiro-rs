use shiro_rs_capi::*;
use std::ptr::{null, null_mut};

#[test]
fn complete_report_collections_preserve_bits_order_empty_rows_and_independent_lifetimes() {
    let bits = [0x8000_0000, 0x7fc1_2345, 0x7f80_0000, 0xff80_0000, 1];
    let values = bits.map(f32::from_bits);
    let info = ShiroRsIterationInfo {
        iteration: usize::MAX,
        temperature: f32::from_bits(0x7fc5_4321),
        mean_log_likelihood: f32::NEG_INFINITY,
    };
    // SAFETY: All input ranges initialized, owners unique, outputs independent.
    unsafe {
        let mut row = null_mut();
        let mut empty = null_mut();
        assert_eq!(
            shiro_rs_array_f32_create(values.as_ptr(), values.len(), &mut row),
            0
        );
        assert_eq!(shiro_rs_array_f32_create(null(), 0, &mut empty), 0);
        let rows = [row.cast_const(), empty.cast_const(), row.cast_const()];
        let mut report = null_mut();
        assert_eq!(
            shiro_rs_iteration_report_create(&info, rows.as_ptr(), rows.len(), &mut report),
            0
        );
        assert_eq!(shiro_rs_array_f32_release(&mut row), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut empty), 0);
        let mut blank = null_mut();
        assert_eq!(
            shiro_rs_iteration_report_create(&info, null(), 0, &mut blank),
            0
        );
        let inputs = [report.cast_const(), blank.cast_const(), report.cast_const()];
        let mut collection = null_mut();
        assert_eq!(
            shiro_rs_iteration_reports_create(inputs.as_ptr(), inputs.len(), &mut collection),
            0
        );
        assert_eq!(shiro_rs_iteration_report_release(&mut report), 0);
        assert_eq!(shiro_rs_iteration_report_release(&mut blank), 0);
        let mut cloned = null_mut();
        assert_eq!(shiro_rs_iteration_reports_clone(collection, &mut cloned), 0);
        assert_eq!(shiro_rs_iteration_reports_release(&mut collection), 0);
        let mut length = 99;
        assert_eq!(shiro_rs_iteration_reports_length(cloned, &mut length), 0);
        assert_eq!(length, 3);
        let mut snapshots = Vec::new();
        for index in 0..length {
            let mut snapshot = null_mut();
            assert_eq!(
                shiro_rs_iteration_reports_get(cloned, index, &mut snapshot),
                0
            );
            snapshots.push(snapshot);
        }
        let mut retained = snapshots[0];
        assert_eq!(
            shiro_rs_iteration_reports_get(cloned, length, &mut retained),
            2
        );
        assert_eq!(retained, snapshots[0]);
        let invalid = [null()];
        let mut retained_collection = cloned;
        assert_eq!(
            shiro_rs_iteration_reports_create(invalid.as_ptr(), 1, &mut retained_collection),
            1
        );
        assert_eq!(retained_collection, cloned);
        assert_eq!(
            shiro_rs_iteration_report_create(&info, null(), 1, &mut retained),
            1
        );
        assert_eq!(retained, snapshots[0]);
        assert_eq!(shiro_rs_iteration_reports_release(&mut cloned), 0);
        assert_eq!(shiro_rs_iteration_reports_release(&mut cloned), 0);
        for (index, mut snapshot) in snapshots.into_iter().enumerate() {
            let mut actual = ShiroRsIterationInfo {
                iteration: 0,
                temperature: 0.0,
                mean_log_likelihood: 0.0,
            };
            assert_eq!(shiro_rs_iteration_report_info(snapshot, &mut actual), 0);
            assert_eq!(actual.iteration, info.iteration);
            assert_eq!(actual.temperature.to_bits(), info.temperature.to_bits());
            assert_eq!(
                actual.mean_log_likelihood.to_bits(),
                info.mean_log_likelihood.to_bits()
            );
            let mut count = 99;
            assert_eq!(
                shiro_rs_iteration_report_file_count(snapshot, &mut count),
                0
            );
            assert_eq!(count, if index == 1 { 0 } else { 3 });
            for row_index in 0..count {
                let mut row = null_mut();
                assert_eq!(
                    shiro_rs_iteration_report_get_file(snapshot, row_index, &mut row),
                    0
                );
                let mut count = 99;
                assert_eq!(shiro_rs_array_f32_length(row, &mut count), 0);
                assert_eq!(count, if row_index == 1 { 0 } else { bits.len() });
                let mut actual = vec![0.0; count];
                assert_eq!(
                    shiro_rs_array_f32_copy(row, 0, actual.as_mut_ptr(), count),
                    0
                );
                assert_eq!(
                    actual
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>(),
                    if row_index == 1 {
                        vec![]
                    } else {
                        bits.to_vec()
                    }
                );
                assert_eq!(shiro_rs_array_f32_release(&mut row), 0);
            }
            assert_eq!(shiro_rs_iteration_report_release(&mut snapshot), 0);
        }
        assert_eq!(
            shiro_rs_iteration_reports_create(null(), 0, &mut collection),
            0
        );
        assert_eq!(
            shiro_rs_iteration_reports_length(collection, &mut length),
            0
        );
        assert_eq!(length, 0);
        assert_eq!(shiro_rs_iteration_reports_release(&mut collection), 0);
    }
}
