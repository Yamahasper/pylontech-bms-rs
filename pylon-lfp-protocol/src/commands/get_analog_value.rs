use log::{error, trace};
use zerocopy::FromBytes;

use crate::types::{ChangeFlags, MilliAmpere, MilliAmpereHours, MilliVolt, Temperature};

/// Errors encountered while parsing a [AnalogValueResponse]
#[derive(Debug)]
pub enum AnalogValueParseError {
    InvalidInput,
}
impl<T: embedded_io::Error> From<AnalogValueParseError> for crate::Error<T> {
    fn from(value: AnalogValueParseError) -> Self {
        match value {
            AnalogValueParseError::InvalidInput => crate::Error::InvalidInput,
        }
    }
}

/// Response payload of a "_get analog value_" command
///
/// Containing flags and measurement data for one or multiple battery packs.
pub struct AnalogValueResponse<'a> {
    /// [PackData] buffer
    buf: &'a [u8],
    pub flags: ChangeFlags,
    /// The total number of packs ([PackData]) reported in this response
    pack_count: u8,
}

/// Measurement data for a pack returned by a "_get analog value_" command
///
/// This can be obtained from a [AnalogValueResponse::get_pack].
#[derive(Debug)]
pub struct PackData<'a> {
    /// Cell voltages
    pub cell_voltages: &'a [MilliVolt],
    /// Temperatures reported for this pack
    pub temperatures: &'a [Temperature],
    /// Current total pack current
    pub pack_current: MilliAmpere,
    /// Current total pack voltage
    pub pack_voltage: MilliVolt,
    /// Remaining pack capacity
    ///
    /// _Note_: It is unclear if this is the currently stored capacity,
    /// or if its the current remaining capacity when fully charged.
    // TODO Clarify the above by experimental validation.
    pub pack_remaining: MilliAmpereHours,
    /// `User-Defined` field
    ///
    /// This is specified to be always `2`. _(?!)_
    pub user_defined: u8,
    /// Total capacity of the pack
    pub total_capacity: MilliAmpereHours,
    /// Cycles of the pack
    pub cell_cycles: u16,
    /// The length in bytes of this PackData
    len_bytes: usize,
}

impl PackData<'_> {
    fn from_bytes(buf: &'_ [u8]) -> Result<PackData<'_>, AnalogValueParseError> {
        if buf.is_empty() {
            return Err(AnalogValueParseError::InvalidInput);
        }

        // Voltages
        let (volt_count, rest) = buf.split_at(1);
        let volt_count = volt_count[0] as usize;
        let (volts, rest) = <[MilliVolt]>::ref_from_prefix_with_elems(rest, volt_count)
            .map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Temperatures
        let (temp_count, rest) = rest.split_at(1);
        let temp_count = temp_count[0] as usize;
        let (temps, rest) = <[Temperature]>::ref_from_prefix_with_elems(rest, temp_count)
            .map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Pack current
        let (pack_current, rest) =
            MilliAmpere::read_from_prefix(rest).map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Pack voltage
        let (pack_voltage, rest) =
            MilliVolt::read_from_prefix(rest).map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Pack remaining
        let (pack_remaining, rest) = MilliAmpereHours::read_from_prefix(rest)
            .map_err(|_| AnalogValueParseError::InvalidInput)?;

        // User-defined field
        let user_defined = *rest.first().ok_or(AnalogValueParseError::InvalidInput)?;
        let rest = rest.get(1..).ok_or(AnalogValueParseError::InvalidInput)?;

        // Total capacity
        let (total_capacity, rest) = MilliAmpereHours::read_from_prefix(rest)
            .map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Cell cycles
        let cell_cycles_be = [rest[0], rest[1]];
        let rest = &rest[2..];
        let cell_cycles = u16::from_be_bytes(cell_cycles_be);

        let len_bytes = buf.len() - rest.len();

        Ok(PackData {
            cell_voltages: volts,
            temperatures: temps,
            pack_current,
            pack_voltage,
            pack_remaining,
            user_defined,
            total_capacity,
            cell_cycles,
            len_bytes,
        })
    }
    fn len(&self) -> usize {
        self.len_bytes
    }
}
impl<'a> AnalogValueResponse<'a> {
    pub fn from_bytes(buf: &'a [u8]) -> Result<AnalogValueResponse<'a>, AnalogValueParseError> {
        if buf.len() < 2 {
            return Err(AnalogValueParseError::InvalidInput);
        }
        let (info, rest) = buf.split_at(2);

        let flags = ChangeFlags::read_from_bytes(&info[..1])
            .map_err(|_| AnalogValueParseError::InvalidInput)?;
        let pack_count = info[1];

        Ok(AnalogValueResponse {
            buf: rest,
            flags,
            pack_count,
        })
    }
    pub fn get_pack_count(&self) -> u8 {
        self.pack_count
    }
    /// Get [PackData] by number
    ///
    /// Indexed starting at `0`.
    pub fn get_pack(&self, pack_number: u8) -> Result<PackData<'_>, AnalogValueParseError> {
        if pack_number >= self.get_pack_count() {
            error!(
                "Analog value response has only {} packs, tried to get {}",
                self.get_pack_count(),
                pack_number
            );
            return Err(AnalogValueParseError::InvalidInput);
        }

        let mut rest = self.buf;
        for i in 0..pack_number {
            let pack = PackData::from_bytes(rest).inspect_err(|_| {
                error!("Failed to traverse pack data while parsing pack {i} in get analog value response");
            })?;
            trace!(
                "Parsed {} bytes of pack data payload for pack {}",
                pack.len(),
                i
            );
            rest = &rest[pack.len()..];
        }
        PackData::from_bytes(rest)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Frame, commands::get_analog_value::AnalogValueResponse, frame::MAX_UNENCODED_PAYLOAD_LEN,
    };
    /// Get the payload from the response in the specification example
    fn payload_from_spec(info_buf: &mut [u8]) -> &[u8] {
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

        let packet =
            Frame::decode(&mut (PACKET.as_slice()), info_buf).expect("Error decoding packet");
        packet.info
    }
    #[test]
    fn parse_flags_and_pack_count() {
        let mut info_buf = [0u8; MAX_UNENCODED_PAYLOAD_LEN];

        let payload = payload_from_spec(&mut info_buf);

        let analog_value_response = AnalogValueResponse::from_bytes(payload)
            .expect("Failed to parse analog value response from payload");

        assert!(analog_value_response.flags.switch_change());
        assert!(analog_value_response.flags.alarm_change());
        assert_eq!(analog_value_response.get_pack_count(), 1);
    }
    #[test]
    fn decode_packet_data() {
        let mut info_buf = [0u8; MAX_UNENCODED_PAYLOAD_LEN];

        let payload = payload_from_spec(&mut info_buf);

        let analog_value_response = AnalogValueResponse::from_bytes(payload)
            .expect("Failed to parse analog value response from payload");

        assert_eq!(analog_value_response.get_pack_count(), 1);

        let pack = analog_value_response
            .get_pack(0)
            .expect("Failed to parse PackData");
        assert_eq!(pack.cell_voltages.len(), 15);
        assert_eq!(pack.temperatures.len(), 5);
        assert_eq!(pack.cell_voltages[0].get(), 3397);
        assert_eq!(pack.cell_voltages[14].get(), 3402);
        assert_eq!(pack.temperatures[0].kelvin(), 301.1);
        assert_eq!(pack.temperatures[4].kelvin(), 302.1);
        assert_eq!(pack.pack_current.get(), 0);
        assert_eq!(pack.pack_voltage.get(), 50981);
        assert_eq!(pack.pack_remaining.get(), 49000);
        assert_eq!(pack.total_capacity.get(), 50000);
        assert_eq!(pack.cell_cycles, 2);
    }
}
