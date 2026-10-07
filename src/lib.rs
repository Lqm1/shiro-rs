//! Phoneme-to-speech alignment toolkit, reimplemented from SHIRO.
//! The complete upstream workflow is under development.
#![forbid(unsafe_code)]
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
pub mod untying;
