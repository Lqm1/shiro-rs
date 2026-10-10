use super::*;
#[napi(js_name = "rawfloat_read")]
pub fn binding_rawfloat_read(
    bytes: napi::bindgen_prelude::Uint8Array,
    maximum_samples: f64,
) -> napi::Result<napi::bindgen_prelude::Float32Array> {
    let storage_bytes = bytes.to_vec();
    (crate::api::rawfloat::rawfloat_read(&storage_bytes, checked_usize(maximum_samples)?))
        .map(<napi::bindgen_prelude::Float32Array>::from)
}
#[napi(js_name = "rawfloat_write")]
pub fn binding_rawfloat_write(
    values: napi::bindgen_prelude::Float32Array,
) -> napi::Result<napi::bindgen_prelude::Uint8Array> {
    let storage_values = values.to_vec();
    (crate::api::rawfloat::rawfloat_write(&storage_values))
        .map(<napi::bindgen_prelude::Uint8Array>::from)
}
