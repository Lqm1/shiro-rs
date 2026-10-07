use shiro_rs::rawfloat;
use std::io::{self, Read, Write};
struct Pieces {
    bytes: std::io::Cursor<Vec<u8>>,
    size: usize,
}
impl Read for Pieces {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let n = buffer.len().min(self.size);
        self.bytes.read(&mut buffer[..n])
    }
}
#[test]
fn bit_patterns_chunk_boundaries_and_budgets() {
    let patterns = [
        0, 0x80000000, 0x3f800000, 0xbf800000, 0x7f800000, 0xff800000, 0x7fc01234, 1,
    ];
    let values: Vec<f32> = (0..9000)
        .map(|i| f32::from_bits(patterns[i % patterns.len()]))
        .collect();
    let mut encoded = Vec::new();
    rawfloat::write(&mut encoded, &values).unwrap();
    for size in [1, 3, 5, 17, 4096, 16384] {
        let mut source = Pieces {
            bytes: std::io::Cursor::new(encoded.clone()),
            size,
        };
        let decoded = rawfloat::read(&mut source, values.len()).unwrap();
        assert_eq!(
            decoded.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            values.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
        assert_eq!(source.read(&mut [0; 4]).unwrap(), 0);
    }
    for count in 1..=3 {
        assert_eq!(
            rawfloat::read(&encoded[..encoded.len() - count], usize::MAX)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
    assert!(rawfloat::read(encoded.as_slice(), values.len() - 1).is_err());
    assert!(rawfloat::read(&[][..], 0).unwrap().is_empty());
    assert_eq!(&encoded[..4], &0u32.to_le_bytes());
    assert_eq!(&encoded[4..8], &0x80000000u32.to_le_bytes());
}
struct Failing;
impl Read for Failing {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("injected read failure"))
    }
}
impl Write for Failing {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected write failure"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("caller owns flush"))
    }
}
#[test]
fn io_failures_propagate_and_empty_output_does_not_flush() {
    assert!(rawfloat::read(Failing, 1).is_err());
    assert!(rawfloat::write(Failing, &[1.0]).is_err());
    assert!(rawfloat::write(Failing, &[]).is_ok());
}
