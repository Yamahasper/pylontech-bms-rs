use embedded_io::{Read, Write};

use crate::{Error, util};
use core::fmt::Display;
use log::{debug, warn};
use util::*;

/// The maximum size of the ASCII encoded payload in bytes
pub const MAX_ENCODED_PAYLOAD_LEN: usize = 4095;
/// The maximum size of unencoded payload data in bytes that a message can contain
pub const MAX_UNENCODED_PAYLOAD_LEN: usize = MAX_ENCODED_PAYLOAD_LEN / 2;

/// A protocol frame
#[derive(Debug)]
pub struct Frame<'a> {
    /// Protocol version field
    pub ver: Version,
    /// Battery address
    pub adr: u8,
    /// `CID1`
    ///
    /// Control identifier 1
    cid1: Cid1,
    /// `CID2`
    ///
    /// Either a command or a response code.
    pub cid2: Cid2,
    /// `LENGTH`
    ///
    /// Encodes the length of the `INFO` field.
    pub length: InfoLength,
    /// `INFO` in binay form
    ///
    /// The payload of the frame.
    /// Either command data (`COMMAND_INFO`) or
    /// response data (`DATA_INFO`).
    pub info: &'a [u8],
}
impl<'a> Frame<'a> {
    /// The Start of Information flag (`~`)
    const SOI: u8 = 0x7E;
    /// The End of Information flag (Carriage Return (CR) `\r`)
    const EOI: u8 = 0x0D;
    /// Construct a new frame
    ///
    /// `info` has to be the unencoded payload.
    pub fn new(ver: Version, adr: u8, cid2: Cid2, info: &'a [u8]) -> Frame<'a> {
        let length = InfoLength::new(info.len() as u16 * 2);
        Self {
            ver,
            adr,
            cid1: Cid1::BatteryData,
            cid2,
            length,
            info,
        }
    }
    /// Decode a ASCII encoded packet
    ///
    /// Fills the `info_buf` with the decoded (binary) payload.
    /// Returns [Error::InvalidInput] when the `SOI` wasn't encountered as first byte.
    /// Returns [Error::Internal] when the `info_buf` isn't large enough.
    pub fn decode<R: Read>(
        reader: &mut R,
        info_buf: &'a mut [u8],
    ) -> Result<Frame<'a>, Error<R::Error>> {
        let mut soi = [0; 1];
        if reader.read(&mut soi)? != 1 {
            return Err(Error::InvalidInput);
        };
        if soi[0] != Self::SOI {
            return Err(Error::InvalidInput);
        }

        let mut checksum = Checksum::new();

        let mut u8_buf = [0u8; 2];
        let mut u16_buf = [0u8; 4];

        // Decode version
        reader.read_exact(&mut u8_buf)?;
        checksum.update(&u8_buf);
        let ver = Version::decode_hex(&u8_buf)?;
        debug!("Decoded ver {ver}");

        // Decode address
        reader.read_exact(&mut u8_buf)?;
        checksum.update(&u8_buf);
        let adr = u8_from_hex(&u8_buf)?;
        debug!("Decoded adr {adr:#04X}");

        // Decode CID1
        reader.read_exact(&mut u8_buf)?;
        checksum.update(&u8_buf);
        Cid1::decode_hex(&u8_buf)?;
        debug!("CID1 ok");

        // Decode CID2
        reader.read_exact(&mut u8_buf)?;
        checksum.update(&u8_buf);
        let cid2 = ResponseCode::decode_hex(&u8_buf)?;
        debug!("Decoded response code: {cid2:?}");

        // Decode LENGTH
        reader.read_exact(&mut u16_buf)?;
        checksum.update(&u16_buf);
        let length = InfoLength::decode_hex(&u16_buf)?;
        length.validate().map_err(|_| Error::Cecksum)?;
        debug!("Decoded valid payload length: {}", length.length());

        // Return if we can't read the full frame
        if info_buf.len() < length.length() as usize / 2 {
            warn!(
                "Buffer for payload to small ({} < {} ({} hex values))",
                info_buf.len(),
                length.length() / 2,
                length.length()
            );
            return Err(Error::Internal);
        }

        for byte in &mut info_buf[..length.length() as usize / 2] {
            reader.read_exact(&mut u8_buf)?;
            checksum.update(&u8_buf);
            *byte = u8_from_hex(&u8_buf)?;
        }

        // Read CHKSUM
        reader.read_exact(&mut u16_buf)?;
        let chksum = u16_from_hex(&u16_buf)?;
        let calculated_checksum = checksum.finalize();
        debug!("Decoded checksum {chksum}, calculated checksum {calculated_checksum}");
        if chksum != calculated_checksum {
            return Err(Error::Cecksum);
        }

        if cid2.is_err() {
            return Err(Error::Response(cid2));
        }
        Ok(Frame::new(
            ver,
            adr,
            cid2.into(),
            &info_buf[..length.length() as usize],
        ))
    }
    /// Construct a fully assembled ASCII/HEX encoded packet of data
    ///
    /// Returns [Error::InvalidInput] when the payload is to large,
    /// (larger than [MAX_UNENCODED_PAYLOAD_LEN]).
    pub fn encode<W: Write>(&self, out: &mut W) -> Result<(), Error<W::Error>> {
        if self.info.len() > MAX_UNENCODED_PAYLOAD_LEN {
            return Err(Error::InvalidInput);
        }
        let Cid2::Command(cmd) = self.cid2 else {
            return Err(Error::Internal);
        };
        let mut chksum = Checksum::new();

        // write SOI
        out.write_all(&[Self::SOI])?;

        // encode version
        let ver = self.ver.encode_hex();
        chksum.update(&ver);
        out.write(&ver)?;

        // encode address
        let adr = self.encode_adr();
        chksum.update(&adr);
        out.write(&adr)?;

        // encode CID1
        let cid1 = self.cid1.encode_hex();
        chksum.update(&cid1);
        out.write(&cid1)?;

        // encode CID2
        let cid2 = cmd.encode_hex();
        chksum.update(&cid2);
        out.write(&cid2)?;

        // encode LENGTH
        let len = self.length.encode_hex();
        chksum.update(&len);
        out.write(&len)?;

        // write data
        for byte in self.info {
            let encoded = u8_encode_hex(*byte);
            chksum.update(&encoded);
            out.write_all(&encoded)?;
        }

        // write checksum
        let chksum = chksum.finalize();
        out.write_all(u16_encode_hex(chksum).as_slice())?;

        // write EOI
        out.write_all(&[Self::EOI])?;

        Ok(())
    }

    fn encode_adr(&self) -> [u8; 2] {
        u8_encode_hex(self.adr)
    }
}

/// Encoded protocol version
#[derive(Debug)]
pub struct Version(u8);
impl Version {
    /// Create a new [Version] from `major` and `minor`
    ///
    /// _Note:_ `major` and `minor` are only stored in 4bit.
    /// Values greater than `15` will be truncated.
    pub fn new(major: u8, minor: u8) -> Self {
        Self((major << 4) ^ (minor & 0b1111))
    }
    pub fn major(&self) -> u8 {
        self.0 >> 4
    }
    pub fn minor(&self) -> u8 {
        self.0 & 0b1111
    }
    pub fn encode_hex(&self) -> [u8; 2] {
        u8_encode_hex(self.0)
    }
    pub fn decode_hex(ascii: &[u8; 2]) -> Result<Self, DecodeError> {
        Ok(Self(u8_from_hex(ascii)?))
    }
}
impl Default for Version {
    fn default() -> Self {
        // TODO: Implement feature flags for different protocol versions.
        //       For now we simply default to the implemented RS232 protocol version.
        Self::new(
            crate::RS232_PROTOCOL_VERSION_MAJOR,
            crate::RS232_PROTOCOL_VERSION_MINOR,
        )
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "v{}.{}", self.major(), self.minor())
    }
}

/// `CID1` control identifier
///
/// RS232 (ver. 2.8) and RS485 (ver. 3.3) protocols
/// only specify one `CID1` which is `BatteryData`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
enum Cid1 {
    BatteryData = 0x46,
}
impl Cid1 {
    fn encode_hex(&self) -> [u8; 2] {
        u8_encode_hex(*self as u8)
    }
    pub fn decode_hex(ascii: &[u8; 2]) -> Result<Cid1, DecodeError> {
        let value = u8_from_hex(ascii)?;
        if value == Self::BatteryData as u8 {
            Ok(Self::BatteryData)
        } else {
            Err(DecodeError::UnknownVariant)
        }
    }
}

/// `CID2` control identifier
///
/// Eiter a command code or a response code.
#[derive(Debug)]
pub enum Cid2 {
    Command(CommandCode),
    Response(ResponseCode),
}
impl From<CommandCode> for Cid2 {
    fn from(value: CommandCode) -> Self {
        Self::Command(value)
    }
}
impl From<ResponseCode> for Cid2 {
    fn from(value: ResponseCode) -> Self {
        Self::Response(value)
    }
}

/// `CID2` command codes (for both RS232 and RS485 protocol)
///
/// Some of the command codes are only available in the RS232 protocol version.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum CommandCode {
    /// Get analog value, fixed point
    GetAnalogValue = 0x42,
    /// Get alarm info
    GetAlarmInfo = 0x44,
    /// Get system parameter, fixed point
    GetSystemParameter = 0x47,
    /// Get protocol version
    GetProtocolVersion = 0x4f,
    /// Get manufacturer info
    GetManufacturerInfo = 0x51,
    /// Get quantity of pack (RS232, ver. 2.8)
    GetQuantityOfPack = 0x90,
    /// Set communication (baud) rate (RS232)
    SetCommunicationRate = 0x91,
    /// Get charge / discharge management info
    GetCharge = 0x92,
    /// Get Serial Number (SN) of battery
    GetSerialNumber = 0x93,
    /// Setup charge / discharge management info
    SetChargeInfo = 0x94,
    /// Turn off (since ver. 2.8)
    TurnOff = 0x95,
    /// Get firmware info
    GetFirmwareInfo = 0x96,
    /// Control command (user-defined) (RS232, ver. 2.8)
    ControlCommand = 0x99,
}
impl CommandCode {
    fn encode_hex(&self) -> [u8; 2] {
        u8_encode_hex(*self as u8)
    }
    pub fn decode_hex(ascii: &[u8; 2]) -> Result<CommandCode, DecodeError> {
        let value = u8_from_hex(ascii)?;
        let cmd = match value {
            0x42 => CommandCode::GetAnalogValue,
            0x44 => CommandCode::GetAlarmInfo,
            0x47 => CommandCode::GetSystemParameter,
            0x4f => CommandCode::GetProtocolVersion,
            0x51 => CommandCode::GetManufacturerInfo,
            0x90 => CommandCode::GetQuantityOfPack,
            0x91 => CommandCode::SetCommunicationRate,
            0x92 => CommandCode::GetCharge,
            0x93 => CommandCode::GetSerialNumber,
            0x94 => CommandCode::SetChargeInfo,
            0x95 => CommandCode::TurnOff,
            0x96 => CommandCode::GetFirmwareInfo,
            0x99 => CommandCode::ControlCommand,
            _ => return Err(DecodeError::UnknownVariant),
        };
        Ok(cmd)
    }
}

/// `CID2` response codes
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum ResponseCode {
    /// Success
    Normal = 0x00,
    /// Version error
    VerError = 0x01,
    /// Frame checksum error
    ChksumErr = 0x02,
    /// Lenght field checksum error
    LChksumErr = 0x03,
    /// CID2 invalid
    Cid2Err = 0x04,
    /// Command format is invalid
    CommandFormatErr = 0x05,
    /// info (payload) data invalid
    InvalidData = 0x06,
    /// Address error
    AdrErr = 0x90,
    /// Internal communication error
    ///
    /// Issued when communication between master and slave pack fails.
    CommunicationErr = 0x91,
}
impl ResponseCode {
    fn is_err(&self) -> bool {
        !self.is_ok()
    }
    fn is_ok(&self) -> bool {
        *self == ResponseCode::Normal
    }
    pub fn decode_hex(ascii: &[u8; 2]) -> Result<ResponseCode, DecodeError> {
        let value = u8_from_hex(ascii)?;
        let cmd = match value {
            0x00 => ResponseCode::Normal,
            0x01 => ResponseCode::VerError,
            0x02 => ResponseCode::ChksumErr,
            0x03 => ResponseCode::LChksumErr,
            0x04 => ResponseCode::Cid2Err,
            0x05 => ResponseCode::CommandFormatErr,
            0x06 => ResponseCode::InvalidData,
            0x90 => ResponseCode::AdrErr,
            0x91 => ResponseCode::CommunicationErr,
            _ => return Err(DecodeError::UnknownVariant),
        };
        Ok(cmd)
    }
}

/// Encoded length of the `INFO` field
///
/// This datatype encodes the lenght of the frame payload (`INFO` field).
/// The encoded value holds the lengh (referred to as `LENID` in the spec)
/// and a checksum (reffered to as `LCHKSUM` in the spec).
#[derive(PartialEq, Eq, Debug)]
pub struct InfoLength(u16);

impl InfoLength {
    /// Encode a new `INFO` length of `length`
    fn new(length: u16) -> Self {
        debug_assert!(length <= 0b1111_1111_1111);

        let nibble1 = length & 0b1111;
        let nibble2 = (length & 0b1111_0000) >> 4;
        let nibble3 = (length & 0b1111_0000_0000) >> 8;

        let sum = nibble1 + nibble2 + nibble3;

        let checksum = (!(sum % 16) & 0b1111) + 1;

        Self((checksum << 12) + length)
    }
    fn encode_hex(&self) -> [u8; 4] {
        u16_encode_hex(self.0)
    }
    fn decode_hex(ascii: &[u8; 4]) -> Result<Self, DecodeError> {
        Ok(Self(u16_from_hex(ascii)?))
    }
    fn validate(&self) -> Result<(), ()> {
        let check = Self::new(self.length());
        if self != &check { Err(()) } else { Ok(()) }
    }
    fn length(&self) -> u16 {
        self.0 & 0b1111_1111_1111
    }
}

/// Checksum that can be updated multiple times before finalizing
struct Checksum {
    acc: u32,
}
impl Checksum {
    /// Create a new [Checksum]
    fn new() -> Self {
        Checksum { acc: 0 }
    }
    /// Update the checksum with new data
    fn update(&mut self, data: &[u8]) {
        for value in data {
            self.acc += *value as u32;
        }
    }
    /// Finalize the checksum
    ///
    /// Also resets the internal state for reuse.
    fn finalize(&mut self) -> u16 {
        let checksum = !(self.acc % 65536) + 1;
        self.acc = 0;
        checksum as u16
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Once;
    static INIT: Once = Once::new();

    #[test]
    fn test_info_length() {
        use super::InfoLength;
        const EXPECTED: u16 = 0b1101_0000_0001_0010;
        const INPUT: u16 = 18;
        let length = InfoLength::new(INPUT);
        assert_eq!(length.0, EXPECTED);
    }
    #[test]
    fn test_version_encoding() {
        use super::Version;

        let ver = Version::new(2, 8);
        assert_eq!(ver.major(), 2);
        assert_eq!(ver.minor(), 8);
        assert_eq!(format!("{ver}"), "v2.8");
        assert_eq!(&ver.encode_hex(), b"28");
    }
    #[test]
    fn test_calculate_checksum() {
        use super::Checksum;
        const EXPECTED: u16 = 0xFC71; //Pylontech calculated 0xFC72 for some reason
        const INPUT: &[u8; 16] = b"1203400456ABCEFE";
        let mut chksum = Checksum::new();
        chksum.update(INPUT);
        assert_eq!(chksum.finalize(), EXPECTED);
    }

    #[test]
    fn test_calculate_checksum2() {
        // 7E 32 30 30 31 34 36 34 32 45 30 30 32 30 31 46 44 33 35 0D
        use super::Checksum;
        const EXPECTED: u16 = 0xFD35; //46 44 33 35
        const INPUT: &[u8; 14] = &[
            0x32, 0x30, 0x30, 0x31, 0x34, 0x36, 0x34, 0x32, 0x45, 0x30, 0x30, 0x32, 0x30, 0x31,
        ];
        let mut chksum = Checksum::new();
        chksum.update(INPUT);
        assert_eq!(chksum.finalize(), EXPECTED);
    }
    #[test]
    fn test_encode_get_version() {
        use super::*;

        const EXPECTED: &[u8; 18] = b"~2801464F0000FD91\r";
        let packet = Frame::new(
            Version::default(),
            1,
            CommandCode::GetProtocolVersion.into(),
            &[],
        );

        let mut buf: Vec<u8> = Vec::new();

        packet.encode(&mut buf).expect("Error encoding frame");

        assert_eq!(
            buf,
            EXPECTED,
            "Expected {:?} got {:?}",
            str::from_utf8(EXPECTED),
            str::from_utf8(&buf)
        );
    }
    #[test]
    fn test_encode_get_analog_value() {
        use super::*;

        const EXPECTED: &[u8; 20] = &[
            0x7E, // SOI
            0x32, 0x30, // v2.0
            0x30, 0x31, // adr 01
            0x34, 0x36, // CID1
            0x34, 0x32, // CID2 (GetAnalogValue)
            0x45, 0x30, 0x30, 0x32, // LENGTH (2)
            0x30, 0x31, // INFO
            0x46, 0x44, 0x33, 0x35, // CHKSUM
            0x0D, // EIO
        ];
        let packet = Frame::new(
            Version::new(2, 0),
            1,
            CommandCode::GetAnalogValue.into(),
            &[0x01],
        );

        let mut buf: Vec<u8> = Vec::new();

        packet.encode(&mut buf).expect("Error encoding frame");

        println!("{:?}", str::from_utf8(&buf));
        assert_eq!(
            buf,
            EXPECTED,
            "Expected {:?} got {:?}",
            str::from_utf8(EXPECTED),
            str::from_utf8(&buf)
        );
    }

    #[test]
    fn test_decode_frame1() {
        use super::*;
        INIT.call_once(|| {
            simple_logger::init_with_level(log::Level::Debug).unwrap();
        });

        /// Get Analog Value response from 20 Cell LiFePo
        const PACKET: [u8; 172] = [
            0x7E, 0x32, 0x35, 0x30, 0x31, 0x34, 0x36, 0x30, 0x30, 0x44, 0x30, 0x39, 0x41, 0x30,
            0x30, 0x30, 0x31, 0x31, 0x34, 0x30, 0x44, 0x30, 0x41, 0x30, 0x44, 0x30, 0x41, 0x30,
            0x44, 0x30, 0x42, 0x30, 0x44, 0x30, 0x42, 0x30, 0x44, 0x30, 0x42, 0x30, 0x44, 0x30,
            0x42, 0x30, 0x44, 0x30, 0x43, 0x30, 0x44, 0x30, 0x42, 0x30, 0x44, 0x30, 0x42, 0x30,
            0x44, 0x30, 0x41, 0x30, 0x44, 0x30, 0x43, 0x30, 0x44, 0x30, 0x43, 0x30, 0x44, 0x30,
            0x43, 0x30, 0x44, 0x30, 0x43, 0x30, 0x44, 0x30, 0x43, 0x30, 0x44, 0x30, 0x43, 0x30,
            0x44, 0x30, 0x43, 0x30, 0x44, 0x30, 0x43, 0x30, 0x44, 0x30, 0x43, 0x30, 0x44, 0x30,
            0x43, 0x30, 0x41, 0x30, 0x42, 0x37, 0x37, 0x30, 0x42, 0x37, 0x35, 0x30, 0x42, 0x37,
            0x36, 0x30, 0x42, 0x37, 0x36, 0x30, 0x42, 0x37, 0x38, 0x30, 0x42, 0x37, 0x41, 0x30,
            0x42, 0x37, 0x36, 0x30, 0x42, 0x37, 0x36, 0x30, 0x42, 0x33, 0x43, 0x30, 0x42, 0x34,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x41, 0x31, 0x36, 0x31, 0x45, 0x41, 0x35, 0x30,
            0x34, 0x32, 0x37, 0x31, 0x30, 0x30, 0x30, 0x30, 0x34, 0x32, 0x37, 0x31, 0x30, 0x44,
            0x42, 0x45, 0x35, 0x0D,
        ];

        let mut info_buf = [0u8; MAX_UNENCODED_PAYLOAD_LEN];

        let packet =
            Frame::decode(&mut (PACKET.as_slice()), &mut info_buf).expect("Error decoding packet");

        println!("{packet:#?}");
    }
    #[test]
    fn test_decode_frame2() {
        use super::*;
        INIT.call_once(|| {
            simple_logger::init_with_level(log::Level::Debug).unwrap();
        });

        /// Example response from specification
        const PACKET: [u8; 128] = [
            0x7E, 0x32, 0x30, 0x30, 0x31, 0x34, 0x36, 0x30, 0x30, 0x43, 0x30, 0x36, 0x45, 0x31,
            0x31, 0x30, 0x31, 0x30, 0x46, 0x30, 0x44, 0x34, 0x35, 0x30, 0x44, 0x34, 0x34, 0x30,
            0x44, 0x34, 0x35, 0x30, 0x44, 0x34, 0x34, 0x30, 0x44, 0x34, 0x35, 0x30, 0x44, 0x34,
            0x34, 0x30, 0x44, 0x33, 0x45, 0x30, 0x44, 0x34, 0x35, 0x30, 0x44, 0x34, 0x41, 0x30,
            0x44, 0x34, 0x41, 0x30, 0x44, 0x34, 0x42, 0x30, 0x44, 0x34, 0x41, 0x30, 0x44, 0x34,
            0x41, 0x30, 0x44, 0x34, 0x41, 0x30, 0x44, 0x34, 0x41, 0x30, 0x35, 0x30, 0x42, 0x43,
            0x33, 0x30, 0x42, 0x43, 0x33, 0x30, 0x42, 0x43, 0x33, 0x30, 0x42, 0x43, 0x44, 0x30,
            0x42, 0x43, 0x44, 0x30, 0x30, 0x30, 0x30, 0x43, 0x37, 0x32, 0x35, 0x42, 0x46, 0x36,
            0x38, 0x30, 0x32, 0x43, 0x33, 0x35, 0x30, 0x30, 0x30, 0x30, 0x32, 0x45, 0x35, 0x35,
            0x33, 0x0D,
        ];

        let mut info_buf = [0u8; MAX_UNENCODED_PAYLOAD_LEN];

        let packet =
            Frame::decode(&mut (PACKET.as_slice()), &mut info_buf).expect("Error decoding packet");

        println!("{packet:#?}");
    }
}
