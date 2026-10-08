//! Optional C boundary for complete SHIRO workflows, built on the native modules.
//! Statuses are zero success, one invalid pointer, two invalid range,
//! three native input/I/O/calculation failure and four a caught unwinding panic.
//! Pointer checks cannot prove allocation validity, lifetime or exclusive access.
#![deny(unsafe_op_in_unsafe_fn)]

use std::{
    io,
    panic::{UnwindSafe, catch_unwind},
};

mod buffers;
pub use buffers::*;
mod models;
pub use models::*;

fn range<T>(pointer: *const T, count: usize) -> Result<(), u32> {
    if count > isize::MAX as usize / size_of::<T>().max(1) {
        return Err(2);
    }
    if count != 0 && (pointer.is_null() || !pointer.is_aligned()) {
        return Err(1);
    }
    Ok(())
}

fn boundary<T>(operation: impl FnOnce() -> io::Result<T> + UnwindSafe) -> Result<T, u32> {
    match catch_unwind(operation) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(_)) => Err(3),
        Err(payload) => {
            // A panic payload may itself panic when dropped; it cannot cross this ABI.
            std::mem::forget(payload);
            Err(4)
        }
    }
}

unsafe fn result<T: Copy>(
    output: *mut T,
    operation: impl FnOnce() -> io::Result<T> + UnwindSafe,
) -> u32 {
    if let Err(status) = range(output.cast_const(), 1) {
        return status;
    }
    match boundary(operation) {
        Ok(value) => {
            // SAFETY: Public caller supplies independent aligned exclusively writable storage.
            unsafe { output.write(value) };
            0
        }
        Err(status) => status,
    }
}

/// First revision of SHIRO's additive C interface.
#[unsafe(no_mangle)]
pub extern "C" fn shiro_rs_abi_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    #[test]
    fn panic_payload_destructor_cannot_unwind_through_the_boundary() {
        struct Payload;
        impl Drop for Payload {
            fn drop(&mut self) {
                panic!("panic payload must remain inside the boundary");
            }
        }
        assert_eq!(
            super::boundary::<()>(|| std::panic::panic_any(Payload)),
            Err(4)
        );
        assert_eq!(
            super::boundary::<()>(|| Err(std::io::Error::other("native failure"))),
            Err(3)
        );
    }
}
