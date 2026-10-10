use super::error;
use napi::Error as BindingError;
#[derive(Clone)]
pub struct PhoneMap {
    pub(crate) inner: crate::labels::PhoneMap,
}
impl PhoneMap {
    pub fn new(json: &str) -> Result<Self, BindingError> {
        serde_json::from_str(json)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn json(&self) -> Result<String, BindingError> {
        serde_json::to_string(&self.inner).map_err(error)
    }
    pub fn replace(&mut self, json: &str) -> Result<(), BindingError> {
        let inner = serde_json::from_str(json).map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone)]
pub struct States {
    pub(crate) inner: Vec<crate::labels::State>,
}
impl States {
    pub fn new(json: &str) -> Result<Self, BindingError> {
        serde_json::from_str(json)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn json(&self) -> Result<String, BindingError> {
        serde_json::to_string(&self.inner).map_err(error)
    }
    pub fn replace(&mut self, json: &str) -> Result<(), BindingError> {
        let inner = serde_json::from_str(json).map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone)]
pub struct SegmentationDocument {
    pub(crate) inner: crate::labels::SegmentationDocument,
}
impl SegmentationDocument {
    pub fn new(json: &str) -> Result<Self, BindingError> {
        serde_json::from_str(json)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn json(&self) -> Result<String, BindingError> {
        serde_json::to_string(&self.inner).map_err(error)
    }
    pub fn replace(&mut self, json: &str) -> Result<(), BindingError> {
        let inner = serde_json::from_str(json).map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
#[derive(Clone)]
pub struct ModelDefinition {
    pub(crate) inner: crate::definition::ModelDefinition,
}
impl ModelDefinition {
    pub fn new(json: &str) -> Result<Self, BindingError> {
        serde_json::from_str(json)
            .map(|inner| Self { inner })
            .map_err(error)
    }
    pub fn json(&self) -> Result<String, BindingError> {
        serde_json::to_string(&self.inner).map_err(error)
    }
    pub fn replace(&mut self, json: &str) -> Result<(), BindingError> {
        let inner = serde_json::from_str(json).map_err(error)?;
        self.inner = inner;
        Ok(())
    }
    pub fn cloned(&self) -> Self {
        self.clone()
    }
}
