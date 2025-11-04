#![no_std]
use embedded_io::Read;
use embedded_io::Write;

mod types;
mod util;

use types::*;

pub struct PylontechBms<U: Read + Write> {
    uart: U,
}

impl<U: Read + Write> PylontechBms<U> {
    pub fn new(uart: U) -> Self {
        PylontechBms { uart }
    }

    pub fn get_protocol_version(&mut self) {
        let _packet = Frame::new(
            Version::default(),
            1,
            CommandCode::GetProtocolVersion.into(),
            &[],
        );
        todo!()
    }
}
