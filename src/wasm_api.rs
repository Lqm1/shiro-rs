//! Optional browser and Node.js interfaces using owned in-memory inputs.
//! Returned arrays and document owners are independent copies. Native validation
//! errors become JavaScript exceptions; a WebAssembly panic is not recoverable.
mod alignment;
mod audio;
mod data;
mod documents;
mod features;
mod initialization;
mod isolation;
mod labels;
mod loading;
mod models;
mod phones;
mod rawfloat;
mod training;
mod untying;
mod utterances;
pub use alignment::*;
pub use audio::*;
pub use data::*;
pub use documents::*;
pub use features::*;
pub use initialization::*;
pub use isolation::*;
pub use labels::*;
pub use loading::*;
pub use models::*;
pub use phones::*;
pub use rawfloat::*;
pub use training::*;
pub use untying::*;
pub use utterances::*;

use wasm_bindgen::prelude::*;

fn error(value: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&value.to_string())
}
