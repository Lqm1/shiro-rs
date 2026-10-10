use super::*;
#[napi(js_name = "batch_feature_options")]
pub fn binding_batch_feature_options(value: u32) -> napi::Result<FeatureOptions> {
    (crate::api::batch::batch_feature_options(value)).map(|value| FeatureOptions {
        inner: std::cell::RefCell::new(Some(value)),
    })
}
#[napi(js_name = "batch_extract_file")]
pub fn binding_batch_extract_file(
    stem: String,
    options: &BatchOptions,
    extractor: &Extractor,
    #[napi(ts_arg_type = "() => number")] uniform: napi::bindgen_prelude::Unknown<'_>,
    env: napi::Env,
) -> napi::Result<BatchOutputs> {
    (crate::api::host::batch_extract_file(
        &stem,
        options
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?,
        extractor
            .inner
            .try_borrow()
            .map_err(borrowed)?
            .as_ref()
            .ok_or_else(consumed)?,
        &crate::callbacks::Function::new(env, uniform)?,
    ))
    .map(|value| BatchOutputs {
        inner: std::cell::RefCell::new(Some(value)),
    })
}
