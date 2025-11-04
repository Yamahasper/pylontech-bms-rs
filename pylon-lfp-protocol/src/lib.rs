#![cfg_attr(not(test), no_std)]
use core::fmt::Display;

use embedded_io::Read;
use embedded_io::Write;

mod types;
mod util;

pub use types::ResponseCode;
use types::*;

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
        let _packet = Frame::new(
            Version::default(),
            1,
            CommandCode::GetProtocolVersion.into(),
            &[],
        )
        .map_err(|_| Error::Internal)?;
        todo!()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error<T: embedded_io::Error> {
    Response(ResponseCode),
    Transport(T),
    Internal,
}

impl<T: embedded_io::Error> Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Response(response_code) => write!(f, "{response_code:?}"),
            Error::Transport(e) => write!(f, "Transport error: {e}"),
            Error::Internal => write!(f, "Internal error"),
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
