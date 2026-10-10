use super::{Model, SegmentationDocument, error};
use crate::untying::{self, Assignment as NativeAssignment, UntiedModel as NativeUntiedModel};
use napi::Error as BindingError;
#[derive(Clone, Copy)]
pub struct Assignment {
    pub state: usize,
    pub file: usize,
    pub segment: usize,
}
impl Assignment {
    pub(super) fn native(&self) -> NativeAssignment {
        NativeAssignment {
            state: self.state,
            file: self.file,
            segment: self.segment,
        }
    }
}
impl Assignment {
    pub fn new(state: usize, file: usize, segment: usize) -> Self {
        Self {
            state,
            file,
            segment,
        }
    }
    pub fn cloned(&self) -> Self {
        *self
    }
}
#[derive(Clone, Default)]
pub struct Assignments {
    inner: Vec<NativeAssignment>,
}
impl Assignments {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<Assignment> {
        self.inner.get(index).map(|value| Assignment {
            state: value.state,
            file: value.file,
            segment: value.segment,
        })
    }
    pub fn push(&mut self, value: &Assignment) {
        self.inner.push(value.native());
    }
    pub fn replace(&mut self, index: usize, value: &Assignment) -> Result<(), BindingError> {
        *self
            .inner
            .get_mut(index)
            .ok_or_else(|| error("assignment index out of range"))? = value.native();
        Ok(())
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone)]
pub struct UntiedModel {
    inner: NativeUntiedModel,
}
impl UntiedModel {
    pub fn new(
        model: &Model,
        segmentation: &SegmentationDocument,
        assignments: &Assignments,
    ) -> Self {
        Self {
            inner: NativeUntiedModel {
                model: model.inner.clone(),
                segmentation: segmentation.inner.clone(),
                assignments: assignments.inner.clone(),
            },
        }
    }
    pub fn model(&self) -> Model {
        Model {
            inner: self.inner.model.clone(),
        }
    }
    pub fn set_model(&mut self, value: &Model) {
        self.inner.model = value.inner.clone();
    }
    pub fn segmentation(&self) -> SegmentationDocument {
        SegmentationDocument {
            inner: self.inner.segmentation.clone(),
        }
    }
    pub fn set_segmentation(&mut self, value: &SegmentationDocument) {
        self.inner.segmentation = value.inner.clone();
    }
    pub fn assignments(&self) -> Assignments {
        Assignments {
            inner: self.inner.assignments.clone(),
        }
    }
    pub fn set_assignments(&mut self, value: &Assignments) {
        self.inner.assignments = value.inner.clone();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn write_summary(&self) -> Result<Vec<u8>, BindingError> {
        let mut bytes = Vec::new();
        self.inner.write_summary(&mut bytes).map_err(error)?;
        Ok(bytes)
    }
}
impl Model {
    pub fn untie(&self, document: &SegmentationDocument) -> Result<UntiedModel, BindingError> {
        untying::untie(&self.inner, &document.inner)
            .map(|inner| UntiedModel { inner })
            .map_err(error)
    }
}
