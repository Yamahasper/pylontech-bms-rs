use embedded_io::Write;

use crate::{Error, util};
use core::fmt::Display;
use util::*;

const MAX_ENCODED_PAYLOAD_LEN: usize = 4095;

pub struct Frame<'a> {
    /// Protocol version field
    ver: Version,
    /// Battery address
    adr: u8,
    /// `CID1`
    ///
    /// Control identifier 1
    cid1: Cid1,
    /// `CID2`
    ///
    /// Either a command or a response code.
    cid2: Cid2,
    /// `LENGTH`
    ///
    /// Encodes the length of the `INFO` field.
    length: InfoLength,
    /// `INFO` in ASCII encoded form
    ///
    /// The payload of the frame.
    /// Either command data (`COMMAND_INFO`) or
    /// response data (`DATA_INFO`).
    info: &'a [u8],
}
impl<'a> Frame<'a> {
    /// The Start of Information flag (`~`)
    const SOI: u8 = 0x7E;
    /// The End of Information flag (Carriage Return (CR) `\r`)
    const EOI: u8 = 0x0D;
    /// Construct a new frame
    ///
    /// `info` has to be the ASCII encoded payload.
    /// Returns an error when info is larger than [MAX_ENCODED_PAYLOAD_LEN].
    pub fn new(ver: Version, adr: u8, cid2: Cid2, info: &'a [u8]) -> Result<Frame<'a>, ()> {
        if info.len() > MAX_ENCODED_PAYLOAD_LEN {
            return Err(());
        }
        let length = InfoLength::new(info.len() as u16);
        Ok(Self {
            ver,
            adr,
            cid1: Cid1::BatteryData,
            cid2,
            length,
            info,
        })
    }
    /// Decodes a ASCII encoded packet
    pub fn decode(ascii: &'a [u8]) -> Result<Frame<'a>, ()> {
        todo!()
    }
    /// Construct a fully assembled ASCII/HEX encoded packet of data
    pub fn encode<W: Write>(&self, out: &mut W) -> Result<(), Error<W::Error>> {
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
        chksum.update(self.info);
        out.write_all(self.info)?;

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
pub struct Version(pub u8);
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
}
impl Default for Version {
    fn default() -> Self {
        // TODO: Implement feature flags for different protocol versions.
        //       For now we simply default to the latest RS232 version (2.8).
        Self::new(2, 8)
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
}

/// Encoded length of the `INFO` field
///
/// This datatype encodes the lenght of the frame payload (`INFO` field).
/// The encoded value holds the lengh (referred to as `LENID` in the spec)
/// and a checksum (reffered to as `LCHKSUM` in the spec).
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
        )
        .expect("Error setting up frame");

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
            &[0x30, 0x31],
        )
        .expect("Error setting up frame");

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
}
