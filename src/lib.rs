//! Phoneme-to-speech alignment toolkit, reimplemented from SHIRO.
//! The complete upstream workflow is under development.
#![deny(unsafe_code)]
#![cfg_attr(not(feature = "c-api"), forbid(unsafe_code))]
#[cfg(feature = "c-api")]
#[allow(unsafe_code)]
pub mod c_api;
pub use ciglet_rs as dsp;
pub use liblrhsmm_rs as hsmm;
pub mod alignment;
pub mod audio;
pub mod batch;
pub mod dataset;
pub mod definition;
pub mod features;
pub mod index;
pub mod initialization;
pub mod labels;
pub mod phonemap;
pub mod rawfloat;
pub mod segmentation;
pub mod training;
pub mod untying;
pub mod utterances;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
pub mod wasm_api;
