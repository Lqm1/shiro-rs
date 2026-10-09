use super::error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn rawfloat_read(bytes: &[u8], maximum_samples: usize) -> Result<Vec<f32>, JsValue> {
    crate::rawfloat::read(bytes, maximum_samples).map_err(error)
}

#[wasm_bindgen]
pub fn rawfloat_write(values: &[f32]) -> Result<Vec<u8>, JsValue> {
    let mut bytes = Vec::new();
    crate::rawfloat::write(&mut bytes, values).map_err(error)?;
    Ok(bytes)
}
