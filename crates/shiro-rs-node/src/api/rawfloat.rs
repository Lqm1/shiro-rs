use super::error;
use napi::Error as BindingError;
pub fn rawfloat_read(bytes: &[u8], maximum_samples: usize) -> Result<Vec<f32>, BindingError> {
    crate::rawfloat::read(bytes, maximum_samples).map_err(error)
}
pub fn rawfloat_write(values: &[f32]) -> Result<Vec<u8>, BindingError> {
    let mut bytes = Vec::new();
    crate::rawfloat::write(&mut bytes, values).map_err(error)?;
    Ok(bytes)
}
