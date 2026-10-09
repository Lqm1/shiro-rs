use super::error;
use wasm_bindgen::prelude::*;

// Full JSON replacement retains every native field, including flattened unknown
// attributes, optional state fields and additional metadata entries.
macro_rules! document {
    ($name:ident, $native:ty) => {
        #[wasm_bindgen]
        #[derive(Clone)]
        pub struct $name {
            pub(crate) inner: $native,
        }

        #[wasm_bindgen]
        impl $name {
            #[wasm_bindgen(constructor)]
            pub fn new(json: &str) -> Result<Self, JsValue> {
                serde_json::from_str(json)
                    .map(|inner| Self { inner })
                    .map_err(error)
            }

            pub fn json(&self) -> Result<String, JsValue> {
                serde_json::to_string(&self.inner).map_err(error)
            }

            pub fn replace(&mut self, json: &str) -> Result<(), JsValue> {
                let inner = serde_json::from_str(json).map_err(error)?;
                self.inner = inner;
                Ok(())
            }

            pub fn cloned(&self) -> Self {
                self.clone()
            }
        }
    };
}

document!(PhoneMap, crate::labels::PhoneMap);
document!(States, Vec<crate::labels::State>);
document!(SegmentationDocument, crate::labels::SegmentationDocument);
document!(ModelDefinition, crate::definition::ModelDefinition);
