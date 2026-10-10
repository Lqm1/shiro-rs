use super::*;
#[napi(js_name = "feature_frame_count")]
pub fn binding_feature_frame_count(
    bytes: napi::bindgen_prelude::BigInt,
    dimensions: f64,
) -> napi::Result<f64> {
    (crate::api::phones::feature_frame_count(checked_u64(&bytes)?, checked_usize(dimensions)?))
        .map(|value| value as f64)
}
