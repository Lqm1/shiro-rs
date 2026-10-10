use super::{
    AlignmentOptions, BatchOptions, BatchOutputs, Dataset, Datasets, Model, SegmentationDocument,
    error,
};
use crate::batch;
use crate::callbacks::Function;
use pyo3::PyErr as BindingError;
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Clone)]
pub struct SptkPrograms {
    pub frame: String,
    pub mfcc: String,
    pub delta: String,
}
impl SptkPrograms {
    pub fn new() -> Self {
        Self {
            frame: "frame".into(),
            mfcc: "mfcc".into(),
            delta: "delta".into(),
        }
    }
}
/// Select native computation or an explicitly configured external extractor.
#[derive(Clone)]
pub struct Extractor {
    inner: batch::Extractor,
}
impl Extractor {
    pub fn native(preset: u32) -> Result<Self, BindingError> {
        let preset = match preset {
            0 => batch::Preset::Mfcc12Da16k,
            1 => batch::Preset::Mfcc12Dae16k,
            2 => batch::Preset::Plpcc12Da16k,
            _ => return Err(error("invalid extraction preset")),
        };
        Ok(Self {
            inner: batch::Extractor::Native(preset),
        })
    }
    pub fn sptk(programs: &SptkPrograms) -> Self {
        Self {
            inner: batch::Extractor::Sptk(batch::SptkPrograms {
                frame: PathBuf::from(&programs.frame),
                mfcc: PathBuf::from(&programs.mfcc),
                delta: PathBuf::from(&programs.delta),
            }),
        }
    }
    pub fn lua(interpreter: String, script: String, executable_directory: String) -> Self {
        Self {
            inner: batch::Extractor::Lua {
                interpreter: interpreter.into(),
                script: script.into(),
                executable_directory: executable_directory.into(),
            },
        }
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
pub fn batch_extract_file(
    stem: &str,
    options: &BatchOptions,
    extractor: &Extractor,
    uniform: &Function,
) -> Result<BatchOutputs, BindingError> {
    let mut callback_error = None;
    let result = batch::extract_file(
        Path::new(stem),
        &batch::Options {
            audio: options.audio().native()?,
            input_extension: options.input_extension(),
        },
        &extractor.inner,
        || {
            if callback_error.is_some() {
                return f32::NAN;
            }
            match uniform.call0(&()).and_then(|value| {
                value
                    .as_f64()
                    .ok_or_else(|| error("uniform callback must return a number"))
            }) {
                Ok(value) => value as f32,
                Err(error) => {
                    callback_error = Some(error);
                    f32::NAN
                }
            }
        },
    );
    if let Some(error) = callback_error {
        return Err(error);
    }
    result
        .map(|outputs| BatchOutputs {
            raw: outputs.raw.to_string_lossy().into_owned(),
            parameters: outputs.parameters.to_string_lossy().into_owned(),
            mfcc: outputs.mfcc.map(|path| path.to_string_lossy().into_owned()),
        })
        .map_err(error)
}
impl Model {
    pub fn read_file(path: &str) -> Result<Self, BindingError> {
        Self::read(&fs::read(path).map_err(pyo3::PyErr::from)?)
    }
    pub fn write_file(&self, path: &str, encoding: u32) -> Result<(), BindingError> {
        fs::write(path, self.write_with_encoding(encoding)?).map_err(pyo3::PyErr::from)
    }
    pub fn align_document_files(
        &self,
        document: &SegmentationDocument,
        options: &AlignmentOptions,
    ) -> Result<SegmentationDocument, BindingError> {
        crate::alignment::align_document(&self.inner, &document.inner, options.native()?)
            .map(|inner| SegmentationDocument { inner })
            .map_err(error)
    }
}
impl Dataset {
    pub fn load_files(
        document: &SegmentationDocument,
        model: &Model,
        maximum_frames: usize,
    ) -> Result<Self, BindingError> {
        crate::dataset::load(&document.inner, &model.inner, maximum_frames)
            .map(|inner| Self { inner })
            .map_err(error)
    }
}
impl Datasets {
    pub fn load_training_paths(
        document: &SegmentationDocument,
        model: &Model,
        maximum_frames: usize,
        isolated: bool,
    ) -> Result<Self, BindingError> {
        crate::dataset::load_training_files(&document.inner, &model.inner, maximum_frames, isolated)
            .map(|inner| Self { inner })
            .map_err(error)
    }
}
