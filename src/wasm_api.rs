//! Optional browser and Node.js interfaces using owned in-memory inputs.
//! Returned arrays and document owners are independent copies. Native validation
//! errors become JavaScript exceptions; a WebAssembly panic is not recoverable.
mod documents;
mod labels;
mod rawfloat;
pub use documents::*;
pub use labels::*;
pub use rawfloat::*;

use wasm_bindgen::prelude::*;

fn error(value: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&value.to_string())
}
