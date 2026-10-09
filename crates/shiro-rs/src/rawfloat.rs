//! Headerless little-endian binary32 streams used by SHIRO tools.
use std::io::{self, Read, Write};

/// Decode a borrowed stream without seeking. Reject a partial final scalar
/// and enforce a sample budget before allocating further output storage.
pub fn read<R: Read>(mut reader: R, maximum_samples: usize) -> io::Result<Vec<f32>> {
    let mut values = Vec::new();
    let mut buffer = [0u8; 16_384];
    let mut pending = 0;
    loop {
        let count = match reader.read(&mut buffer[pending..]) {
            Ok(0) => {
                if pending != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "rawfloat stream ends with a partial scalar",
                    ));
                }
                return Ok(values);
            }
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        let used = pending + count;
        let scalars = used / 4;
        if scalars > maximum_samples.saturating_sub(values.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "rawfloat stream exceeds the sample budget",
            ));
        }
        values.try_reserve(scalars).map_err(io::Error::other)?;
        for bytes in buffer[..scalars * 4].chunks_exact(4) {
            values.push(f32::from_le_bytes(
                bytes.try_into().expect("four-byte scalar"),
            ));
        }
        pending = used % 4;
        buffer.copy_within(scalars * 4..used, 0);
    }
}

/// Preserve all binary32 bit patterns. The caller owns and flushes the stream.
pub fn write<W: Write>(mut writer: W, values: &[f32]) -> io::Result<()> {
    let mut buffer = [0u8; 16_384];
    for chunk in values.chunks(buffer.len() / 4) {
        for (&value, bytes) in chunk.iter().zip(buffer.chunks_exact_mut(4)) {
            bytes.copy_from_slice(&value.to_le_bytes());
        }
        writer.write_all(&buffer[..chunk.len() * 4])?;
    }
    Ok(())
}
