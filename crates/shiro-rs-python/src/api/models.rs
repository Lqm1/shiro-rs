use super::{ModelDefinition, error};
use pyo3::PyErr as BindingError;
use shiro_rs::hsmm::serial::{DecodeLimits, ModelEncoding};
use shiro_rs::hsmm::{
    Duration as NativeDuration, GaussianMixture as NativeGaussian, Model as NativeModel,
    Stream as NativeStream,
};
#[derive(Clone, Default)]
pub struct Duration {
    inner: NativeDuration,
}
impl Duration {
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
    pub fn set_values(&mut self, values: &[f32]) -> Result<(), BindingError> {
        let [mean, variance, variance_floor] = values else {
            return Err(error("duration values require three entries"));
        };
        self.inner.mean = *mean;
        self.inner.variance = *variance;
        self.inner.variance_floor = *variance_floor;
        Ok(())
    }
    pub fn set_constraints(&mut self, values: &[i32]) -> Result<(), BindingError> {
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
#[derive(Clone, Default)]
pub struct Durations {
    pub(super) inner: Vec<NativeDuration>,
}
impl Durations {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<Duration> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| Duration { inner })
    }
    pub fn push(&mut self, value: &Duration) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &Duration) -> Result<(), BindingError> {
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
pub struct Gaussian {
    inner: NativeGaussian,
}
impl Gaussian {
    pub fn new(components: usize, dimensions: usize) -> Result<Self, BindingError> {
        NativeGaussian::new(components, dimensions)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn dimensions(&self) -> usize {
        self.inner.dimensions
    }
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
    ) -> Result<(), BindingError> {
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
#[derive(Clone, Default)]
pub struct Gaussians {
    pub(super) inner: Vec<NativeGaussian>,
}
impl Gaussians {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<Gaussian> {
        self.inner
            .get(index)
            .cloned()
            .map(|inner| Gaussian { inner })
    }
    pub fn push(&mut self, value: &Gaussian) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &Gaussian) -> Result<(), BindingError> {
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
pub struct Stream {
    inner: NativeStream,
}
impl Stream {
    pub fn new(
        emissions: usize,
        components: usize,
        dimensions: usize,
    ) -> Result<Self, BindingError> {
        NativeStream::new(emissions, components, dimensions)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn emissions(&self) -> usize {
        self.inner.mixtures.len()
    }
    pub fn weight(&self) -> Vec<f32> {
        vec![self.inner.weight]
    }
    pub fn set_weight(&mut self, values: &[f32]) -> Result<(), BindingError> {
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
    pub fn set_emission(&mut self, index: usize, value: &Gaussian) -> Result<(), BindingError> {
        *self
            .inner
            .mixtures
            .get_mut(index)
            .ok_or_else(|| error("emission index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn replace(&mut self, weight: &[f32], emissions: &Gaussians) -> Result<(), BindingError> {
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
#[derive(Clone, Default)]
pub struct Streams {
    pub(super) inner: Vec<NativeStream>,
}
impl Streams {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn length(&self) -> usize {
        self.inner.len()
    }
    pub fn get(&self, index: usize) -> Option<Stream> {
        self.inner.get(index).cloned().map(|inner| Stream { inner })
    }
    pub fn push(&mut self, value: &Stream) {
        self.inner.push(value.inner.clone());
    }
    pub fn replace(&mut self, index: usize, value: &Stream) -> Result<(), BindingError> {
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
pub struct Model {
    pub(crate) inner: NativeModel,
}
impl Model {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn streams(&self) -> usize {
        self.inner.streams.len()
    }
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
    pub fn set_stream(&mut self, index: usize, value: &Stream) -> Result<(), BindingError> {
        *self
            .inner
            .streams
            .get_mut(index)
            .ok_or_else(|| error("stream index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn set_duration(&mut self, index: usize, value: &Duration) -> Result<(), BindingError> {
        *self
            .inner
            .durations
            .get_mut(index)
            .ok_or_else(|| error("duration index out of range"))? = value.inner.clone();
        Ok(())
    }
    pub fn replace(
        &mut self,
        streams: &Streams,
        durations: &Durations,
    ) -> Result<(), BindingError> {
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
    pub fn validate(&self) -> Result<(), BindingError> {
        self.inner.validate().map_err(error)
    }
    pub fn dimensions(&self) -> Result<Vec<u32>, BindingError> {
        crate::dataset::dimensions(&self.inner)
            .map(|values| values.into_iter().map(|value| value as u32).collect())
            .map_err(error)
    }
    pub fn read(bytes: &[u8]) -> Result<Self, BindingError> {
        Self::read_with_limits(bytes, DecodeLimits::default().max_array_entries)
    }
    pub fn read_with_limits(bytes: &[u8], max_array_entries: usize) -> Result<Self, BindingError> {
        NativeModel::read_with_limits(bytes, DecodeLimits { max_array_entries })
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn read_prefix(bytes: &[u8]) -> Result<DecodedModel, BindingError> {
        Self::read_prefix_with_limits(bytes, DecodeLimits::default().max_array_entries)
    }
    pub fn read_prefix_with_limits(
        bytes: &[u8],
        max_array_entries: usize,
    ) -> Result<DecodedModel, BindingError> {
        let mut reader = std::io::Cursor::new(bytes);
        let inner =
            NativeModel::read_prefix_with_limits(&mut reader, DecodeLimits { max_array_entries })
                .map_err(error)?;
        Ok(DecodedModel {
            inner,
            position: reader.position(),
        })
    }
    pub fn write(&self) -> Result<Vec<u8>, BindingError> {
        self.write_with_encoding(0)
    }
    pub fn write_with_encoding(&self, code: u32) -> Result<Vec<u8>, BindingError> {
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
pub struct DecodedModel {
    inner: NativeModel,
    position: u64,
}
impl DecodedModel {
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
impl ModelDefinition {
    pub fn build(&self) -> Result<Model, BindingError> {
        self.inner
            .build()
            .map(|inner| Model { inner })
            .map_err(error)
    }
}
