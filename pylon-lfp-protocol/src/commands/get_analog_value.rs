use log::{error, trace};
use zerocopy::FromBytes;

use crate::types::{
    Ampere, AmpereHours, ChangeFlags, Temperature, Volt,
    exponents::{DECI, MILLI},
};

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
pub struct PackData<
    'a,
    const CELL_VOLTAGE_EXP: i8 = MILLI,
    const TOTAL_VOLTAGE_EXP: i8 = MILLI,
    const CURRENT_EXP: i8 = MILLI,
    const AMP_HOUR_EXP: i8 = MILLI,
    const TEMP_EXP: i8 = DECI,
> {
    /// Cell voltages
    pub cell_voltages: &'a [Volt<CELL_VOLTAGE_EXP>],
    /// Temperatures reported for this pack
    pub temperatures: &'a [Temperature<TEMP_EXP>],
    /// Current total pack current
    pub pack_current: Ampere<CURRENT_EXP>,
    /// Current total pack voltage
    pub pack_voltage: Volt<TOTAL_VOLTAGE_EXP>,
    /// Current remaining charge
    pub pack_remaining: AmpereHours<AMP_HOUR_EXP>,
    /// `User-Defined` field
    ///
    /// This is specified to be always `2`. _(?!)_
    pub user_defined: u8,
    /// Total capacity of the pack
    pub total_capacity: AmpereHours<AMP_HOUR_EXP>,
    /// Cycles of the pack
    pub cell_cycles: u16,
    /// The length in bytes of this PackData
    len_bytes: usize,
}

impl<
    'a,
    const CELL_VOLTAGE_EXP: i8,
    const TOTAL_VOLTAGE_EXP: i8,
    const CURRENT_EXP: i8,
    const AMP_HOUR_EXP: i8,
    const TEMP_EXP: i8,
> PackData<'a, CELL_VOLTAGE_EXP, TOTAL_VOLTAGE_EXP, CURRENT_EXP, AMP_HOUR_EXP, TEMP_EXP>
{
    fn from_bytes(buf: &'a [u8]) -> Result<Self, AnalogValueParseError> {
        if buf.is_empty() {
            return Err(AnalogValueParseError::InvalidInput);
        }

        // Voltages
        let (volt_count, rest) = buf.split_at(1);
        let volt_count = volt_count[0] as usize;
        let (volts, rest) = <[Volt<_>]>::ref_from_prefix_with_elems(rest, volt_count)
            .map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Temperatures
        let (temp_count, rest) = rest.split_at(1);
        let temp_count = temp_count[0] as usize;
        let (temps, rest) = <[Temperature<TEMP_EXP>]>::ref_from_prefix_with_elems(rest, temp_count)
            .map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Pack current
        let (pack_current, rest) =
            Ampere::read_from_prefix(rest).map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Pack voltage
        let (pack_voltage, rest) =
            Volt::read_from_prefix(rest).map_err(|_| AnalogValueParseError::InvalidInput)?;

        // Pack remaining
        let (pack_remaining, rest) =
            AmpereHours::read_from_prefix(rest).map_err(|_| AnalogValueParseError::InvalidInput)?;

        // User-defined field
        let user_defined = *rest.first().ok_or(AnalogValueParseError::InvalidInput)?;
        let rest = rest.get(1..).ok_or(AnalogValueParseError::InvalidInput)?;

        // Total capacity
        let (total_capacity, rest) =
            AmpereHours::read_from_prefix(rest).map_err(|_| AnalogValueParseError::InvalidInput)?;

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
    /// Get the number of packs reported by this response
    pub fn get_pack_count(&self) -> u8 {
        self.pack_count
    }
    /// Get [PackData] by number
    ///
    /// Indexed starting at `0`.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use pylon_lfp_protocol::commands::{AnalogValueResponse, PackData};
    /// # fn get_pack_example(payload: &[u8]) {
    /// let response = AnalogValueResponse::from_bytes(payload)
    ///     .expect("Failed to parse analog value response from payload");
    ///
    /// for i in 0..response.get_pack_count() {
    ///    // Using the default exponents here by specifying the type as `PackData<'_>`.
    ///    let pack: PackData<'_> = response
    ///        .get_pack(i)
    ///        .expect("Failed to parse PackData");
    ///    println!("Pack {i}: {:?}", pack);
    /// }
    /// # }
    /// ```
    pub fn get_pack<
        const CELL_VOLTAGE_EXP: i8,
        const TOTAL_VOLTAGE_EXP: i8,
        const CURRENT_EXP: i8,
        const AMP_HOUR_EXP: i8,
    >(
        &self,
        pack_number: u8,
    ) -> Result<
        PackData<'_, CELL_VOLTAGE_EXP, TOTAL_VOLTAGE_EXP, CURRENT_EXP, AMP_HOUR_EXP>,
        AnalogValueParseError,
    > {
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
            let pack: PackData<'_> = PackData::from_bytes(rest).inspect_err(|_| {
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
        Frame,
        commands::{PackData, get_analog_value::AnalogValueResponse},
        frame::MAX_UNENCODED_PAYLOAD_LEN,
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

        let pack: PackData<'_> = analog_value_response
            .get_pack(0)
            .expect("Failed to parse PackData");
        assert_eq!(pack.cell_voltages.len(), 15);
        assert_eq!(pack.temperatures.len(), 5);
        assert_eq!(pack.cell_voltages[0].get_raw(), 3397);
        assert_eq!(pack.cell_voltages[14].get_raw(), 3402);
        assert_eq!(pack.temperatures[0].kelvin(), 301.1);
        assert_eq!(pack.temperatures[4].kelvin(), 302.1);
        assert_eq!(pack.pack_current.get_raw(), 0);
        assert_eq!(pack.pack_voltage.get_raw(), 50981);
        assert_eq!(pack.pack_remaining.get_raw(), 49000);
        assert_eq!(pack.total_capacity.get_raw(), 50000);
        assert_eq!(pack.cell_cycles, 2);
    }
}
