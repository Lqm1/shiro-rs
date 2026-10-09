use super::models::collection;
use super::{Model, States, error};
use liblrhsmm_rs::data::{Jump, ObservationStreamData};
use liblrhsmm_rs::{
    Dataset as NativeDataset, Observation as NativeObservation, Segmentation as NativeSegmentation,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct ObservationStream {
    inner: ObservationStreamData<f32>,
}
#[wasm_bindgen]
impl ObservationStream {
    #[wasm_bindgen(constructor)]
    pub fn new(dimensions: usize, values: &[f32]) -> Self {
        Self {
            inner: ObservationStreamData {
                dimensions,
                values: values.to_vec(),
            },
        }
    }
    #[wasm_bindgen(getter)]
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
collection!(
    ObservationStreams,
    ObservationStream,
    ObservationStreamData<f32>
);

#[wasm_bindgen]
#[derive(Clone)]
pub struct Observation {
    pub(crate) inner: NativeObservation,
}
#[wasm_bindgen]
impl Observation {
    #[wasm_bindgen(constructor)]
    pub fn new(frames: usize, dimensions: &[u32]) -> Result<Self, JsValue> {
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
    ) -> Result<Self, JsValue> {
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
    ) -> Result<Self, JsValue> {
        let dimensions = crate::dataset::dimensions(&model.inner).map_err(error)?;
        crate::dataset::read_observation(bytes, &dimensions, maximum_frames)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    #[wasm_bindgen(getter)]
    pub fn frames(&self) -> usize {
        self.inner.frames
    }
    #[wasm_bindgen(getter)]
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
    pub fn set_stream(&mut self, index: usize, value: &ObservationStream) -> Result<(), JsValue> {
        let mut inner = self.inner.clone();
        *inner
            .streams
            .get_mut(index)
            .ok_or_else(|| error("observation stream index out of range"))? = value.inner.clone();
        inner.validate().map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn replace(&mut self, frames: usize, streams: &ObservationStreams) -> Result<(), JsValue> {
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
    pub fn validate(&self) -> Result<(), JsValue> {
        self.inner.validate().map_err(error)
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn write(&self) -> Result<Vec<u8>, JsValue> {
        let mut bytes = Vec::new();
        self.inner.write_to(&mut bytes).map_err(error)?;
        Ok(bytes)
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JumpGroup {
    inner: Vec<Jump>,
}
#[wasm_bindgen]
impl JumpGroup {
    #[wasm_bindgen(constructor)]
    pub fn new(deltas: &[i32], probabilities: &[f32]) -> Result<Self, JsValue> {
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
    #[wasm_bindgen(getter)]
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

#[wasm_bindgen]
#[derive(Clone)]
pub struct Segmentation {
    pub(crate) inner: NativeSegmentation,
}
#[wasm_bindgen]
impl Segmentation {
    #[wasm_bindgen(constructor)]
    pub fn new(streams: usize, segments: usize) -> Result<Self, JsValue> {
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
    pub fn from_states(states: &States, model: &Model) -> Result<Self, JsValue> {
        crate::dataset::read_segmentation(&states.inner, &model.inner)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    #[wasm_bindgen(getter)]
    pub fn streams(&self) -> usize {
        self.inner.output_states.len()
    }
    #[wasm_bindgen(getter)]
    pub fn segments(&self) -> usize {
        self.inner.boundaries.len()
    }
    pub fn boundaries(&self) -> Vec<i32> {
        self.inner.boundaries.clone()
    }
    pub fn duration_states(&self) -> Vec<i32> {
        self.inner.duration_states.clone()
    }
    pub fn output_states(&self, stream: usize) -> Result<Vec<i32>, JsValue> {
        self.inner
            .output_states
            .get(stream)
            .cloned()
            .ok_or_else(|| error("output stream index out of range"))
    }
    pub fn set_boundaries(&mut self, values: &[i32]) -> Result<(), JsValue> {
        if values.len() != self.segments() {
            return Err(error("boundary count differs"));
        }
        self.inner.boundaries.copy_from_slice(values);
        Ok(())
    }
    pub fn set_duration_states(&mut self, values: &[i32]) -> Result<(), JsValue> {
        if values.len() != self.segments() {
            return Err(error("duration state count differs"));
        }
        self.inner.duration_states.copy_from_slice(values);
        Ok(())
    }
    pub fn set_output_states(&mut self, stream: usize, values: &[i32]) -> Result<(), JsValue> {
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
    pub fn set_outgoing(&mut self, state: usize, jumps: &JumpGroup) -> Result<(), JsValue> {
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
    pub fn validate(&self) -> Result<(), JsValue> {
        self.inner.validate().map_err(error)
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn write(&self) -> Result<Vec<u8>, JsValue> {
        let mut bytes = Vec::new();
        self.inner.write_to(&mut bytes).map_err(error)?;
        Ok(bytes)
    }
}

collection!(Observations, Observation, NativeObservation);
collection!(Segmentations, Segmentation, NativeSegmentation);

/// Complete editable native sample arrays. Pairing is checked by consumers.
#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Dataset {
    pub(crate) inner: NativeDataset,
}

#[wasm_bindgen]
impl Dataset {
    #[wasm_bindgen(constructor)]
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
