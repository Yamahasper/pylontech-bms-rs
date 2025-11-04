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
    ///
    /// Returns the slice containing the packet, or an error when the provided buffer is to small.
    pub fn encode(&self, buf: &'a mut [u8]) -> Result<&'a [u8], ()> {
        todo!()
    }
}

/// Encoded protocol version
pub struct Version(pub u8);
impl Version {
    /// Create a new [Version] from `major` and `minor`
    ///
    /// _Note:_ `major` and `minor` are only stored in 4bit.
    /// Values greater than `15` will be truncated.
    pub fn new(major: u8, minor: u8) -> Self {
        Self((major << 4) & (minor & 0b1111))
    }
    pub fn major(&self) -> u8 {
        self.0 >> 4
    }
    pub fn minor(&self) -> u8 {
        self.0 & 0b1111
    }
}
impl Default for Version {
    fn default() -> Self {
        // TODO: Implement feature flags for different protocol versions.
        //       For now we simply default to the latest RS232 version (2.8).
        Self::new(2, 8)
    }
}

/// `CID1` control identifier
///
/// RS232 (ver. 2.8) and RS485 (ver. 3.3) protocols
/// only specify one `CID1` which is `BatteryData`.
#[non_exhaustive]
enum Cid1 {
    BatteryData = 0x46,
}

/// `CID2` control identifier
///
/// Eiter a command code or a response code.
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

/// `CID2` response codes
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

        let checksum = !(sum % 16) + 1;

        Self((checksum << 12) + length)
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
}
