use super::{Model, States, error};
use pyo3::PyErr as BindingError;
use shiro_rs::hsmm::data::{Jump, ObservationStreamData};
use shiro_rs::hsmm::{
    Dataset as NativeDataset, Observation as NativeObservation, Segmentation as NativeSegmentation,
};
#[derive(Clone)]
pub struct ObservationStream {
    inner: ObservationStreamData<f32>,
}
impl ObservationStream {
    pub fn new(dimensions: usize, values: &[f32]) -> Self {
        Self {
            inner: ObservationStreamData {
                dimensions,
                values: values.to_vec(),
            },
        }
    }
    pub fn dimensions(&self) -> usize {
        self.inner.dimensions
    }
    pub fn values(&self) -> Vec<f32> {
        self.inner.values.clone()
    }
    pub fn replace(&mut self, dimensions: usize, values: &[f32]) {
        self.inner = ObservationStreamData {
            dimensions,
            values: values.to_vec(),
        };
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone, Default)]
pub struct ObservationStreams {
    pub(super) inner: Vec<ObservationStreamData<f32>>,
}
impl ObservationStreams {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<ObservationStream> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| ObservationStream { inner })
    }
    pub fn push(&mut self, value: &ObservationStream) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &ObservationStream) -> Result<(), BindingError> {
        *self
            .inner
            .get_mut(index)
            .ok_or_else(|| error("collection index out of range"))? = value.inner.clone();
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
pub struct Observation {
    pub(crate) inner: NativeObservation,
}
impl Observation {
    pub fn new(frames: usize, dimensions: &[u32]) -> Result<Self, BindingError> {
        let dimensions = dimensions
            .iter()
            .map(|&value| value as usize)
            .collect::<Vec<_>>();
        NativeObservation::new(frames, &dimensions)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn read_rawfloat(
        bytes: &[u8],
        dimensions: &[u32],
        maximum_frames: usize,
    ) -> Result<Self, BindingError> {
        let dimensions = dimensions
            .iter()
            .map(|&value| value as usize)
            .collect::<Vec<_>>();
        crate::dataset::read_observation(bytes, &dimensions, maximum_frames)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn from_model_rawfloat(
        bytes: &[u8],
        model: &Model,
        maximum_frames: usize,
    ) -> Result<Self, BindingError> {
        let dimensions = crate::dataset::dimensions(&model.inner).map_err(error)?;
        crate::dataset::read_observation(bytes, &dimensions, maximum_frames)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn frames(&self) -> usize {
        self.inner.frames
    }
    pub fn streams(&self) -> usize {
        self.inner.streams.len()
    }
    pub fn stream(&self, index: usize) -> Option<ObservationStream> {
        self.inner
            .streams
            .get(index)
            .cloned()
            .map(|inner| ObservationStream { inner })
    }
    pub fn set_stream(
        &mut self,
        index: usize,
        value: &ObservationStream,
    ) -> Result<(), BindingError> {
        let mut inner = self.inner.clone();
        *inner
            .streams
            .get_mut(index)
            .ok_or_else(|| error("observation stream index out of range"))? = value.inner.clone();
        inner.validate().map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn replace(
        &mut self,
        frames: usize,
        streams: &ObservationStreams,
    ) -> Result<(), BindingError> {
        let inner = NativeObservation {
            frames,
            streams: streams.inner.clone(),
        };
        inner.validate().map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn frame(&self, stream: usize, time: usize) -> Option<Vec<f32>> {
        self.inner.frame(stream, time).map(<[f32]>::to_vec)
    }
    pub fn validate(&self) -> Result<(), BindingError> {
        self.inner.validate().map_err(error)
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn write(&self) -> Result<Vec<u8>, BindingError> {
        let mut bytes = Vec::new();
        self.inner.write_to(&mut bytes).map_err(error)?;
        Ok(bytes)
    }
}
#[derive(Clone)]
pub struct JumpGroup {
    inner: Vec<Jump>,
}
impl JumpGroup {
    pub fn new(deltas: &[i32], probabilities: &[f32]) -> Result<Self, BindingError> {
        if deltas.len() != probabilities.len() {
            return Err(error("jump deltas and probabilities differ"));
        }
        Ok(Self {
            inner: deltas
                .iter()
                .zip(probabilities)
                .map(|(&delta, &probability)| Jump { delta, probability })
                .collect(),
        })
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn deltas(&self) -> Vec<i32> {
        self.inner.iter().map(|jump| jump.delta).collect()
    }
    pub fn probabilities(&self) -> Vec<f32> {
        self.inner.iter().map(|jump| jump.probability).collect()
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone)]
pub struct Segmentation {
    pub(crate) inner: NativeSegmentation,
}
impl Segmentation {
    pub fn new(streams: usize, segments: usize) -> Result<Self, BindingError> {
        if streams > i32::MAX as usize
            || segments > i32::MAX as usize
            || streams > isize::MAX as usize / size_of::<Vec<i32>>()
            || segments > isize::MAX as usize / size_of::<Vec<Jump>>()
        {
            return Err(error("segmentation shape exceeds the legacy format"));
        }
        Ok(Self {
            inner: NativeSegmentation::new(streams, segments),
        })
    }
    pub fn from_states(states: &States, model: &Model) -> Result<Self, BindingError> {
        crate::dataset::read_segmentation(&states.inner, &model.inner)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn streams(&self) -> usize {
        self.inner.output_states.len()
    }
    pub fn segments(&self) -> usize {
        self.inner.boundaries.len()
    }
    pub fn boundaries(&self) -> Vec<i32> {
        self.inner.boundaries.clone()
    }
    pub fn duration_states(&self) -> Vec<i32> {
        self.inner.duration_states.clone()
    }
    pub fn output_states(&self, stream: usize) -> Result<Vec<i32>, BindingError> {
        self.inner
            .output_states
            .get(stream)
            .cloned()
            .ok_or_else(|| error("output stream index out of range"))
    }
    pub fn set_boundaries(&mut self, values: &[i32]) -> Result<(), BindingError> {
        if values.len() != self.segments() {
            return Err(error("boundary count differs"));
        }
        self.inner.boundaries.copy_from_slice(values);
        Ok(())
    }
    pub fn set_duration_states(&mut self, values: &[i32]) -> Result<(), BindingError> {
        if values.len() != self.segments() {
            return Err(error("duration state count differs"));
        }
        self.inner.duration_states.copy_from_slice(values);
        Ok(())
    }
    pub fn set_output_states(&mut self, stream: usize, values: &[i32]) -> Result<(), BindingError> {
        let target = self
            .inner
            .output_states
            .get_mut(stream)
            .ok_or_else(|| error("output stream index out of range"))?;
        if target.len() != values.len() {
            return Err(error("output state count differs"));
        }
        target.copy_from_slice(values);
        Ok(())
    }
    pub fn outgoing(&self, state: usize) -> Option<JumpGroup> {
        self.inner
            .outgoing
            .get(state)
            .cloned()
            .map(|inner| JumpGroup { inner })
    }
    pub fn set_outgoing(&mut self, state: usize, jumps: &JumpGroup) -> Result<(), BindingError> {
        let mut inner = self.inner.clone();
        *inner
            .outgoing
            .get_mut(state)
            .ok_or_else(|| error("jump state index out of range"))? = jumps.inner.clone();
        inner.validate().map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn replace(&mut self, value: &Segmentation) {
        self.inner = value.inner.clone();
    }
    pub fn validate(&self) -> Result<(), BindingError> {
        self.inner.validate().map_err(error)
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn write(&self) -> Result<Vec<u8>, BindingError> {
        let mut bytes = Vec::new();
        self.inner.write_to(&mut bytes).map_err(error)?;
        Ok(bytes)
    }
}
#[derive(Clone, Default)]
pub struct Observations {
    pub(super) inner: Vec<NativeObservation>,
}
impl Observations {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<Observation> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| Observation { inner })
    }
    pub fn push(&mut self, value: &Observation) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &Observation) -> Result<(), BindingError> {
        *self
            .inner
            .get_mut(index)
            .ok_or_else(|| error("collection index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone, Default)]
pub struct Segmentations {
    pub(super) inner: Vec<NativeSegmentation>,
}
impl Segmentations {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<Segmentation> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| Segmentation { inner })
    }
    pub fn push(&mut self, value: &Segmentation) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &Segmentation) -> Result<(), BindingError> {
        *self
            .inner
            .get_mut(index)
            .ok_or_else(|| error("collection index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
/// Complete editable native sample arrays. Pairing is checked by consumers.
#[derive(Clone, Default)]
pub struct Dataset {
    pub(crate) inner: NativeDataset,
}
impl Dataset {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn observations(&self) -> Observations {
        Observations {
            inner: self.inner.observations.clone(),
        }
    }
    pub fn segmentations(&self) -> Segmentations {
        Segmentations {
            inner: self.inner.segmentations.clone(),
        }
    }
    pub fn set_observations(&mut self, values: &Observations) {
        self.inner.observations = values.inner.clone();
    }
    pub fn set_segmentations(&mut self, values: &Segmentations) {
        self.inner.segmentations = values.inner.clone();
    }
    pub fn replace(&mut self, observations: &Observations, segmentations: &Segmentations) {
        self.inner = NativeDataset {
            observations: observations.inner.clone(),
            segmentations: segmentations.inner.clone(),
        };
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
