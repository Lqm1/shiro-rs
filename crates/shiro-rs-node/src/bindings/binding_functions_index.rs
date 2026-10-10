use super::*;
#[napi(js_name = "index_append_suffix")]
pub fn binding_index_append_suffix(path: String, suffix: String) -> napi::Result<String> {
    crate::api::index::index_append_suffix(&path, &suffix)
}
