//! Phoneme-to-speech alignment toolkit, reimplemented from SHIRO.
//! The complete upstream workflow is under development.
#![forbid(unsafe_code)]
pub use ciglet_rs as dsp;
pub use liblrhsmm_rs as hsmm;
pub mod audio;
pub mod definition;
pub mod features;
pub mod rawfloat;
