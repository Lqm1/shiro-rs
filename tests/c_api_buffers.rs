#![cfg(feature = "c-api")]
use shiro_rs::c_api::*;
use std::ptr::{null, null_mut};

unsafe fn bytes(owner: *const ShiroRsBytes) -> Vec<u8> {
    // SAFETY: Live readable owner and independent initialized output buffers.
    unsafe {
        let mut length = 99;
        assert_eq!(shiro_rs_bytes_length(owner, &mut length), 0);
        let mut values = vec![0; length];
        assert_eq!(
            shiro_rs_bytes_copy(owner, 0, values.as_mut_ptr(), length),
            0
        );
        values
    }
}

#[test]
fn bytes_are_independent_and_invalid_ranges_retain_outputs() {
    // SAFETY: All owners are unique, with independent initialized arrays and slots.
    unsafe {
        assert_eq!(shiro_rs_abi_version(), 1);
        let mut input = [1, 2, 3, 4];
        let mut owner = null_mut();
        let mut copied = null_mut();
        assert_eq!(
            shiro_rs_bytes_create(input.as_ptr(), input.len(), &mut owner),
            0
        );
        input[0] = 99;
        assert_eq!(bytes(owner), [1, 2, 3, 4]);
        assert_eq!(shiro_rs_bytes_clone(owner, &mut copied), 0);
        assert_eq!(shiro_rs_bytes_release(&mut owner), 0);
        assert!(owner.is_null());
        assert_eq!(shiro_rs_bytes_release(&mut owner), 0);
        assert_eq!(bytes(copied), [1, 2, 3, 4]);
        let mut output = [71, 71];
        assert_eq!(shiro_rs_bytes_copy(copied, 3, output.as_mut_ptr(), 2), 2);
        assert_eq!(output, [71, 71]);
        assert_eq!(
            shiro_rs_bytes_copy(copied, usize::MAX, output.as_mut_ptr(), 2),
            2
        );
        assert_eq!(output, [71, 71]);
        assert_eq!(shiro_rs_bytes_copy(copied, 4, null_mut(), 0), 0);
        assert_eq!(shiro_rs_bytes_copy(copied, 5, null_mut(), 0), 2);
        assert_eq!(shiro_rs_bytes_copy(copied, 0, null_mut(), 1), 1);
        assert_eq!(shiro_rs_bytes_clone(copied, null_mut()), 1);
        assert_eq!(shiro_rs_bytes_length(copied, null_mut()), 1);
        let mut length = 99;
        assert_eq!(shiro_rs_bytes_length(null(), &mut length), 1);
        assert_eq!(length, 99);
        let mut sentinel = copied;
        assert_eq!(shiro_rs_bytes_create(null(), 1, &mut sentinel), 1);
        assert_eq!(shiro_rs_bytes_create(null(), usize::MAX, &mut sentinel), 2);
        assert_eq!(sentinel, copied);
        assert_eq!(
            shiro_rs_bytes_create(input.as_ptr(), input.len(), null_mut()),
            1
        );
        assert_eq!(shiro_rs_bytes_release(&mut copied), 0);
        assert_eq!(shiro_rs_bytes_release(null_mut()), 1);
        assert_eq!(shiro_rs_bytes_create(null(), 0, &mut owner), 0);
        assert!(bytes(owner).is_empty());
        assert_eq!(shiro_rs_bytes_copy(owner, 0, null_mut(), 0), 0);
        assert_eq!(shiro_rs_bytes_release(&mut owner), 0);
    }
}

#[test]
fn rawfloat_roundtrip_preserves_every_binary32_bit_and_source_lifetime() {
    let patterns = [
        0,
        0x8000_0000,
        1,
        0x8000_0001,
        0x3f80_0001,
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_1234,
    ];
    let mut input = patterns.map(f32::from_bits);
    let expected_wire = patterns
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>();
    // SAFETY: Independent typed buffers and unique owners, released exactly once.
    unsafe {
        let mut owner = null_mut();
        let mut copied = null_mut();
        let mut encoded = null_mut();
        let mut decoded = null_mut();
        assert_eq!(
            shiro_rs_array_f32_create(input.as_ptr(), input.len(), &mut owner),
            0
        );
        input.fill(77.0);
        assert_eq!(shiro_rs_array_f32_clone(owner, &mut copied), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut owner), 0);
        let mut length = 99;
        assert_eq!(shiro_rs_array_f32_length(copied, &mut length), 0);
        assert_eq!(length, patterns.len());
        let mut values = [99.0_f32; 8];
        assert_eq!(
            shiro_rs_array_f32_copy(copied, 0, values.as_mut_ptr(), values.len()),
            0
        );
        assert_eq!(values.map(f32::to_bits), patterns);
        assert_eq!(shiro_rs_rawfloat_write_bytes(copied, &mut encoded), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut copied), 0);
        assert_eq!(bytes(encoded), expected_wire);
        assert_eq!(
            shiro_rs_rawfloat_read_bytes(
                expected_wire.as_ptr(),
                expected_wire.len(),
                patterns.len(),
                &mut decoded
            ),
            0
        );
        assert_eq!(
            shiro_rs_array_f32_copy(decoded, 0, values.as_mut_ptr(), values.len()),
            0
        );
        assert_eq!(values.map(f32::to_bits), patterns);
        let mut sentinel = decoded;
        for length in [1, 2, 3, expected_wire.len() - 1] {
            assert_eq!(
                shiro_rs_rawfloat_read_bytes(expected_wire.as_ptr(), length, 99, &mut sentinel),
                3
            );
            assert_eq!(sentinel, decoded);
        }
        assert_eq!(
            shiro_rs_rawfloat_read_bytes(
                expected_wire.as_ptr(),
                expected_wire.len(),
                patterns.len() - 1,
                &mut sentinel
            ),
            3
        );
        assert_eq!(sentinel, decoded);
        assert_eq!(
            shiro_rs_rawfloat_read_bytes(null(), 1, 99, &mut sentinel),
            1
        );
        assert_eq!(
            shiro_rs_rawfloat_read_bytes(null(), usize::MAX, 99, &mut sentinel),
            2
        );
        assert_eq!(
            shiro_rs_rawfloat_read_bytes(
                expected_wire.as_ptr(),
                expected_wire.len(),
                99,
                null_mut()
            ),
            1
        );
        assert_eq!(shiro_rs_rawfloat_write_bytes(decoded, null_mut()), 1);
        let mut byte_sentinel = encoded;
        assert_eq!(shiro_rs_rawfloat_write_bytes(null(), &mut byte_sentinel), 1);
        assert_eq!(byte_sentinel, encoded);
        assert_eq!(shiro_rs_array_f32_release(&mut decoded), 0);
        assert_eq!(shiro_rs_bytes_release(&mut encoded), 0);
        assert_eq!(shiro_rs_rawfloat_read_bytes(null(), 0, 0, &mut decoded), 0);
        assert_eq!(shiro_rs_array_f32_length(decoded, &mut length), 0);
        assert_eq!(length, 0);
        assert_eq!(shiro_rs_array_f32_release(&mut decoded), 0);
    }
}

#[test]
fn scalar_array_range_and_pointer_errors_are_transactional() {
    // SAFETY: Unique live owners and initialized independent caller storage.
    unsafe {
        let input = [1.0, -0.0];
        let mut owner = null_mut();
        assert_eq!(
            shiro_rs_array_f32_create(input.as_ptr(), input.len(), &mut owner),
            0
        );
        let mut output = [71.0, 71.0];
        assert_eq!(shiro_rs_array_f32_copy(owner, 1, output.as_mut_ptr(), 2), 2);
        assert_eq!(
            shiro_rs_array_f32_copy(owner, usize::MAX, output.as_mut_ptr(), 2),
            2
        );
        assert_eq!(output, [71.0, 71.0]);
        assert_eq!(shiro_rs_array_f32_copy(owner, 2, null_mut(), 0), 0);
        assert_eq!(shiro_rs_array_f32_copy(owner, 3, null_mut(), 0), 2);
        assert_eq!(shiro_rs_array_f32_copy(owner, 0, null_mut(), 1), 1);
        let mut sentinel = owner;
        assert_eq!(shiro_rs_array_f32_create(null(), 1, &mut sentinel), 1);
        assert_eq!(
            shiro_rs_array_f32_create(null(), usize::MAX, &mut sentinel),
            2
        );
        assert_eq!(sentinel, owner);
        assert_eq!(shiro_rs_array_f32_clone(owner, null_mut()), 1);
        assert_eq!(shiro_rs_array_f32_length(owner, null_mut()), 1);
        assert_eq!(shiro_rs_array_f32_release(&mut owner), 0);
        assert_eq!(shiro_rs_array_f32_release(&mut owner), 0);
        assert_eq!(shiro_rs_array_f32_release(null_mut()), 1);
        assert_eq!(shiro_rs_array_f32_create(null(), 0, &mut owner), 0);
        let mut length = 99;
        assert_eq!(shiro_rs_array_f32_length(owner, &mut length), 0);
        assert_eq!(length, 0);
        assert_eq!(shiro_rs_array_f32_release(&mut owner), 0);
    }
}
