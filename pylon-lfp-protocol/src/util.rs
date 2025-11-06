use crate::Error;

pub fn u8_encode_hex(value: u8) -> [u8; 2] {
    use embedded_io::Write;
    let mut buf = [0u8; 2];
    let _ = buf.as_mut_slice().write_fmt(format_args!("{:02X}", value));
    buf
}
pub fn u16_encode_hex(value: u16) -> [u8; 4] {
    use embedded_io::Write;
    let mut buf = [0u8; 4];
    let _ = buf.as_mut_slice().write_fmt(format_args!("{:04X}", value));
    buf
}

pub fn u8_from_hex(ascii: &[u8; 2]) -> Result<u8, DecodeError> {
    let string = str::from_utf8(ascii)
        .map_err(|_| DecodeError::Hex)
        .inspect_err(|_| log::debug!("Non ascii value encountered"))?;
    u8::from_str_radix(string, 16).map_err(|_| DecodeError::Hex)
}
pub fn u16_from_hex(ascii: &[u8; 4]) -> Result<u16, DecodeError> {
    let string = str::from_utf8(ascii)
        .map_err(|_| DecodeError::Hex)
        .inspect_err(|_| log::debug!("Non ascii value encountered"))?;
    u16::from_str_radix(string, 16).map_err(|_| DecodeError::Hex)
}

#[derive(Debug)]
pub enum DecodeError {
    /// Not a valid hex encoded value
    Hex,
    /// Error deserializing type from decoded value
    ///
    /// Mostly unsupported control identifier.
    UnknownVariant,
}

impl<T: embedded_io::Error> From<DecodeError> for Error<T> {
    fn from(value: DecodeError) -> Self {
        match value {
            DecodeError::Hex => Self::InvalidInput,
            DecodeError::UnknownVariant => Self::UnsupportedControlIdentifier,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_u8_encode_hex() {
        assert_eq!(&super::u8_encode_hex(0x0C), b"0C");
    }
    #[test]
    fn test_u16_encode_hex() {
        assert_eq!(&super::u16_encode_hex(0x0A02), b"0A02");
    }
    #[test]
    fn test_u8_decode_hex() {
        assert_eq!(super::u8_from_hex(b"0C").unwrap(), 0x0C);
    }
    #[test]
    fn test_u16_decode_hex() {
        assert_eq!(super::u16_from_hex(b"0A02").unwrap(), 0x0A02);
    }
}
