//! C ABI for shiro-rs.
#![deny(unsafe_op_in_unsafe_fn)]
use shiro_rs::*;
pub mod c_api;
pub use c_api::*;
