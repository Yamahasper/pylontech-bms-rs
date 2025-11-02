#![no_std]
use embedded_io::Read;
use embedded_io::Write;

pub struct PylontechBms<U:Read+Write>{
    uart: U,
}

impl<U:Read+Write> PylontechBms<U>{
    pub fn new(uart: U) -> Self{
        PylontechBms{uart}
    }

    pub fn get_protocol_version(&mut self){
        todo!()
    }

}

struct InfoLength(u16);

impl InfoLength{
    fn new(length: u16) -> Self{
        debug_assert!(length <= 0b1111_1111_1111);

        let nibble1 = length & 0b1111;
        let nibble2 = (length & 0b1111_0000) >> 4;
        let nibble3 = (length & 0b1111_0000_0000) >> 8;

        let sum = nibble1 + nibble2 + nibble3;

        let checksum = !(sum %16) + 1;

        Self((checksum << 12) + length)
    }
}

fn calculate_checksum(data: &[u8]) -> u16{
    let sum = data.into_iter().fold(0, | acc, x | acc + *x as u32);
    let checksum = !(sum %65536) + 1;
    checksum as u16
}




#[cfg(test)]
mod tests {
    #[test]
    fn test_info_length(){
        use crate::InfoLength;
        const EXPECTED: u16 = 0b1101_0000_0001_0010;
        const INPUT: u16 = 18;
        let length = InfoLength::new(INPUT);
        assert_eq!(length.0, EXPECTED);
    }
    #[test]
    fn test_calculate_checksum(){
        use crate::calculate_checksum;
        const EXPECTED: u16 = 0xFC71; //Pylontech calculated 0xFC72 for some reason
        const INPUT: &[u8; 16] = b"1203400456ABCEFE";
        assert_eq!(calculate_checksum(INPUT), EXPECTED);
    }

      #[test]
    fn test_calculate_checksum2(){
        // 7E 32 30 30 31 34 36 34 32 45 30 30 32 30 31 46 44 33 35 0D
        use crate::calculate_checksum;
        const EXPECTED: u16 = 0xFD35;  //46 44 33 35
        const INPUT: &[u8; 14] = &[0x32, 0x30, 0x30, 0x31, 0x34, 0x36, 0x34, 0x32, 0x45, 0x30, 0x30, 0x32, 0x30, 0x31];
        assert_eq!(calculate_checksum(INPUT), EXPECTED);
    }
}


