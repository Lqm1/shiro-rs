use super::*;
#[napi(js_name = "label_output_path")]
pub fn binding_label_output_path(filename: String, suffix: String) -> napi::Result<String> {
    Ok(crate::api::labels::label_output_path(&filename, &suffix))
}
