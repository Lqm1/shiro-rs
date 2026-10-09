use super::{ModelDefinition, error};
use shiro_rs::hsmm::serial::{DecodeLimits, ModelEncoding};
use shiro_rs::hsmm::{
    Duration as NativeDuration, GaussianMixture as NativeGaussian, Model as NativeModel,
    Stream as NativeStream,
};
use wasm_bindgen::prelude::*;

macro_rules! collection {
    ($name:ident, $owner:ident, $native:ty) => {
        #[wasm_bindgen]
        #[derive(Clone, Default)]
        pub struct $name {
            pub(super) inner: Vec<$native>,
        }
        #[wasm_bindgen]
        impl $name {
            #[wasm_bindgen(constructor)]
            pub fn new() -> Self {
                Self::default()
            }
            #[wasm_bindgen(getter)]
            pub fn length(&self) -> usize {
                self.inner.len()
            }
            pub fn get(&self, index: usize) -> Option<$owner> {
                self.inner.get(index).cloned().map(|inner| $owner { inner })
            }
            pub fn push(&mut self, value: &$owner) {
                self.inner.push(value.inner.clone());
            }
            pub fn replace(&mut self, index: usize, value: &$owner) -> Result<(), JsValue> {
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
    };
}
pub(super) use collection;

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Duration {
    inner: NativeDuration,
}
#[wasm_bindgen]
impl Duration {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn values(&self) -> Vec<f32> {
        vec![
            self.inner.mean,
            self.inner.variance,
            self.inner.variance_floor,
        ]
    }
    pub fn constraints(&self) -> Vec<i32> {
        vec![
            self.inner.minimum,
            self.inner.maximum,
            self.inner.fixed_mean,
        ]
    }
    pub fn set_values(&mut self, values: &[f32]) -> Result<(), JsValue> {
        let [mean, variance, variance_floor] = values else {
            return Err(error("duration values require three entries"));
        };
        self.inner.mean = *mean;
        self.inner.variance = *variance;
        self.inner.variance_floor = *variance_floor;
        Ok(())
    }
    pub fn set_constraints(&mut self, values: &[i32]) -> Result<(), JsValue> {
        let [minimum, maximum, fixed_mean] = values else {
            return Err(error("duration constraints require three entries"));
        };
        self.inner.minimum = *minimum;
        self.inner.maximum = *maximum;
        self.inner.fixed_mean = *fixed_mean;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
collection!(Durations, Duration, NativeDuration);

#[wasm_bindgen]
#[derive(Clone)]
pub struct Gaussian {
    inner: NativeGaussian,
}
#[wasm_bindgen]
impl Gaussian {
    #[wasm_bindgen(constructor)]
    pub fn new(components: usize, dimensions: usize) -> Result<Self, JsValue> {
        NativeGaussian::new(components, dimensions)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    #[wasm_bindgen(getter)]
    pub fn dimensions(&self) -> usize {
        self.inner.dimensions
    }
    #[wasm_bindgen(getter)]
    pub fn components(&self) -> usize {
        self.inner.weights.len()
    }
    pub fn weights(&self) -> Vec<f32> {
        self.inner.weights.clone()
    }
    pub fn means(&self) -> Vec<f32> {
        self.inner.means.clone()
    }
    pub fn variances(&self) -> Vec<f32> {
        self.inner.variances.clone()
    }
    pub fn variance_floors(&self) -> Vec<f32> {
        self.inner.variance_floors.clone()
    }
    pub fn replace(
        &mut self,
        dimensions: usize,
        weights: &[f32],
        means: &[f32],
        variances: &[f32],
        variance_floors: &[f32],
    ) -> Result<(), JsValue> {
        let inner = NativeGaussian {
            dimensions,
            weights: weights.to_vec(),
            means: means.to_vec(),
            variances: variances.to_vec(),
            variance_floors: variance_floors.to_vec(),
        };
        inner.validate().map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
collection!(Gaussians, Gaussian, NativeGaussian);

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Stream {
    inner: NativeStream,
}
#[wasm_bindgen]
impl Stream {
    #[wasm_bindgen(constructor)]
    pub fn new(emissions: usize, components: usize, dimensions: usize) -> Result<Self, JsValue> {
        NativeStream::new(emissions, components, dimensions)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    #[wasm_bindgen(getter)]
    pub fn emissions(&self) -> usize {
        self.inner.mixtures.len()
    }
    pub fn weight(&self) -> Vec<f32> {
        vec![self.inner.weight]
    }
    pub fn set_weight(&mut self, values: &[f32]) -> Result<(), JsValue> {
        let [weight] = values else {
            return Err(error("stream weight requires one entry"));
        };
        self.inner.weight = *weight;
        Ok(())
    }
    pub fn emission(&self, index: usize) -> Option<Gaussian> {
        self.inner
            .mixtures
            .get(index)
            .cloned()
            .map(|inner| Gaussian { inner })
    }
    pub fn set_emission(&mut self, index: usize, value: &Gaussian) -> Result<(), JsValue> {
        *self
            .inner
            .mixtures
            .get_mut(index)
            .ok_or_else(|| error("emission index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn replace(&mut self, weight: &[f32], emissions: &Gaussians) -> Result<(), JsValue> {
        let [weight] = weight else {
            return Err(error("stream weight requires one entry"));
        };
        let inner = NativeStream {
            weight: *weight,
            mixtures: emissions.inner.clone(),
        };
        inner.validate().map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
collection!(Streams, Stream, NativeStream);

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct Model {
    pub(crate) inner: NativeModel,
}
#[wasm_bindgen]
impl Model {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    #[wasm_bindgen(getter)]
    pub fn streams(&self) -> usize {
        self.inner.streams.len()
    }
    #[wasm_bindgen(getter)]
    pub fn durations(&self) -> usize {
        self.inner.durations.len()
    }
    pub fn stream(&self, index: usize) -> Option<Stream> {
        self.inner
            .streams
            .get(index)
            .cloned()
            .map(|inner| Stream { inner })
    }
    pub fn duration(&self, index: usize) -> Option<Duration> {
        self.inner
            .durations
            .get(index)
            .cloned()
            .map(|inner| Duration { inner })
    }
    pub fn set_stream(&mut self, index: usize, value: &Stream) -> Result<(), JsValue> {
        *self
            .inner
            .streams
            .get_mut(index)
            .ok_or_else(|| error("stream index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn set_duration(&mut self, index: usize, value: &Duration) -> Result<(), JsValue> {
        *self
            .inner
            .durations
            .get_mut(index)
            .ok_or_else(|| error("duration index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn replace(&mut self, streams: &Streams, durations: &Durations) -> Result<(), JsValue> {
        let inner = NativeModel {
            streams: streams.inner.clone(),
            durations: durations.inner.clone(),
        };
        inner.validate().map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
    pub fn validate(&self) -> Result<(), JsValue> {
        self.inner.validate().map_err(error)
    }
    pub fn dimensions(&self) -> Result<Vec<u32>, JsValue> {
        crate::dataset::dimensions(&self.inner)
            .map(|values| values.into_iter().map(|value| value as u32).collect())
            .map_err(error)
    }
    pub fn read(bytes: &[u8]) -> Result<Self, JsValue> {
        Self::read_with_limits(bytes, DecodeLimits::default().max_array_entries)
    }
    pub fn read_with_limits(bytes: &[u8], max_array_entries: usize) -> Result<Self, JsValue> {
        NativeModel::read_with_limits(bytes, DecodeLimits { max_array_entries })
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn read_prefix(bytes: &[u8]) -> Result<DecodedModel, JsValue> {
        Self::read_prefix_with_limits(bytes, DecodeLimits::default().max_array_entries)
    }
    pub fn read_prefix_with_limits(
        bytes: &[u8],
        max_array_entries: usize,
    ) -> Result<DecodedModel, JsValue> {
        let mut reader = std::io::Cursor::new(bytes);
        let inner =
            NativeModel::read_prefix_with_limits(&mut reader, DecodeLimits { max_array_entries })
                .map_err(error)?;
        Ok(DecodedModel {
            inner,
            position: reader.position(),
        })
    }
    pub fn write(&self) -> Result<Vec<u8>, JsValue> {
        self.write_with_encoding(0)
    }
    pub fn write_with_encoding(&self, code: u32) -> Result<Vec<u8>, JsValue> {
        let encoding = match code {
            0 => ModelEncoding::WithVarianceFloors,
            1 => ModelEncoding::WithoutVarianceFloors,
            _ => return Err(error("invalid model encoding")),
        };
        let mut bytes = Vec::new();
        self.inner
            .write_with_encoding(&mut bytes, encoding)
            .map_err(error)?;
        Ok(bytes)
    }
}

#[wasm_bindgen]
pub struct DecodedModel {
    inner: NativeModel,
    position: u64,
}
#[wasm_bindgen]
impl DecodedModel {
    #[wasm_bindgen(getter)]
    pub fn position(&self) -> u64 {
        self.position
    }
    pub fn parameters(&self) -> Model {
        Model {
            inner: self.inner.clone(),
        }
    }
    pub fn into_parameters(self) -> Model {
        Model { inner: self.inner }
    }
}

#[wasm_bindgen]
impl ModelDefinition {
    pub fn build(&self) -> Result<Model, JsValue> {
        self.inner
            .build()
            .map(|inner| Model { inner })
            .map_err(error)
    }
}
