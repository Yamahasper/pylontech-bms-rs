/// Calculates the `CHKSUM` over `data` according to the specification
///
/// Corresponds to the format described in section 2.5 of RS232 v2.8,
/// or section 2.3.3 of RS485 v3.3.
///
/// `data` is the ASCII encoded data included in the checksum.
///
/// ## Example
/// Given the `data` `b"1203400456ABCEFE"`, the calculated checksum is
/// `0xFC71`.
/// The frame constructed from this is `~1203400456ABCEFEFC71\R`.
///
/// _Note:_ The specification erranously calculates the checksum as `0xFC72` for this example.
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let sum = data.iter().fold(0, |acc, x| acc + *x as u32);
    let checksum = !(sum % 65536) + 1;
    checksum as u16
}

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

#[cfg(test)]
mod tests {
    #[test]
    fn test_calculate_checksum() {
        use super::calculate_checksum;
        const EXPECTED: u16 = 0xFC71; //Pylontech calculated 0xFC72 for some reason
        const INPUT: &[u8; 16] = b"1203400456ABCEFE";
        assert_eq!(calculate_checksum(INPUT), EXPECTED);
    }

    #[test]
    fn test_calculate_checksum2() {
        // 7E 32 30 30 31 34 36 34 32 45 30 30 32 30 31 46 44 33 35 0D
        use super::calculate_checksum;
        const EXPECTED: u16 = 0xFD35; //46 44 33 35
        const INPUT: &[u8; 14] = &[
            0x32, 0x30, 0x30, 0x31, 0x34, 0x36, 0x34, 0x32, 0x45, 0x30, 0x30, 0x32, 0x30, 0x31,
        ];
        assert_eq!(calculate_checksum(INPUT), EXPECTED);
    }
    #[test]
    fn test_u8_encode_hex() {
        assert_eq!(&super::u8_encode_hex(0x0C), b"0C");
    }
    #[test]
    fn test_u16_encode_hex() {
        assert_eq!(&super::u16_encode_hex(0x0A02), b"0A02");
    }
}
