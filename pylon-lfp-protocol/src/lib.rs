#![cfg_attr(not(test), no_std)]
use core::fmt::Display;

use embedded_io::Read;
use embedded_io::Write;

mod frame;
pub mod types;
mod util;

pub use frame::{
    Cid2, CommandCode, Frame, InfoLength, MAX_ENCODED_PAYLOAD_LEN, ResponseCode, Version,
};

/// Major version this library intends to implement
const RS232_PROTOCOL_VERSION_MAJOR: u8 = 2;
/// Minor version this library intends to implement
const RS232_PROTOCOL_VERSION_MINOR: u8 = 8;

/// Pylontech RS232 protocol BMS
pub struct PylontechBms<U: Read + Write> {
    uart: U,
}

impl<U: Read + Write> PylontechBms<U> {
    pub fn new(uart: U) -> Self {
        PylontechBms { uart }
    }

    /// Get the protocol version from the BMS
    pub fn get_protocol_version(&mut self) -> Result<Version, Error<U::Error>> {
        let packet = Frame::new(
            Version::default(),
            1,
            CommandCode::GetProtocolVersion.into(),
            &[],
        );
        packet.encode(&mut self.uart)?;
        let mut buf = [0u8; MAX_ENCODED_PAYLOAD_LEN]; // TODO payload might be always 0 length for get version
        let response = Frame::decode(&mut self.uart, &mut buf)?;
        Ok(response.ver)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error<T: embedded_io::Error> {
    /// Error signaled by BMS
    Response(ResponseCode),
    /// Transport layer error
    Transport(T),
    /// Invalid frame received
    InvalidInput,
    /// Bad checksum for received frame
    Cecksum,
    /// Internal error
    ///
    /// Encountered a error while processing data.
    Internal,
    /// Unkonwn control identifier received
    ///
    /// `CID1` or `CID2` doesn't match a known identifier.
    /// This might be due to protocol version mismatch or
    /// misbehaving BMS.
    UnsupportedControlIdentifier,
}

impl<T: embedded_io::Error> Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Response(response_code) => write!(f, "{response_code:?}"),
            Error::Transport(e) => write!(f, "Transport error: {e}"),
            Error::Internal => write!(f, "Internal error"),
            Error::InvalidInput => write!(f, "Invalid input"),
            Error::Cecksum => write!(f, "Checksum error"),
            Error::UnsupportedControlIdentifier => write!(f, "Unsupported control identifier"),
        }
    }
}
impl<T: embedded_io::Error + 'static> core::error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Error::Transport(e) => Some(e),
            _ => None,
        }
    }
}
impl<T: embedded_io::Error> From<T> for Error<T> {
    fn from(value: T) -> Self {
        Self::Transport(value)
    }
}
impl<T: embedded_io::Error> From<embedded_io::WriteFmtError<T>> for Error<T> {
    fn from(value: embedded_io::WriteFmtError<T>) -> Self {
        match value {
            embedded_io::WriteFmtError::FmtError => Error::Internal,
            embedded_io::WriteFmtError::Other(e) => Error::Transport(e),
        }
    }
}
impl<T: embedded_io::Error> From<embedded_io::ReadExactError<T>> for Error<T> {
    fn from(value: embedded_io::ReadExactError<T>) -> Self {
        match value {
            embedded_io::ReadExactError::UnexpectedEof => Error::InvalidInput,
            embedded_io::ReadExactError::Other(e) => Error::Transport(e),
        }
    }
}
