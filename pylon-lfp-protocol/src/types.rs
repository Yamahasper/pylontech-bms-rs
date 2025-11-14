//! Datatypes and units

use core::fmt::Display;
use zerocopy::byteorder::big_endian;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

pub type MilliVolt = Volt<1000>;

/// Voltage
///
/// Holds a voltage in Volt with a factor.
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct Volt<const FACTOR: u32>(big_endian::U16);
impl<const FACTOR: u32> Display for Volt<FACTOR> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if FACTOR == 1000 {
            write!(f, "{}mV", self.0)
        } else {
            write!(f, "{}V", self.get_volt())
        }
    }
}
impl<const FACTOR: u32> Volt<FACTOR> {
    pub fn get_raw(&self) -> u16 {
        self.0.get()
    }
    pub fn get_volt(&self) -> f32 {
        self.get_raw() as f32 / FACTOR as f32
    }
}

/// Current
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct MilliAmpere(big_endian::I16);
impl Display for MilliAmpere {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}mA", self.0)
    }
}
impl MilliAmpere {
    pub fn get(&self) -> i16 {
        self.0.get()
    }
}

/// Electric charge
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct MilliAmpereHours(big_endian::U16);
impl Display for MilliAmpereHours {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}mAh", self.0)
    }
}
impl MilliAmpereHours {
    pub fn get(&self) -> u16 {
        self.0.get()
    }
}

/// Temperature
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct Temperature {
    /// dK (dezi Kelvin) (0.1K)
    d_kelvin: big_endian::U16,
}
impl Temperature {
    /// The temperature in Kelvin
    pub fn kelvin(&self) -> f32 {
        self.d_kelvin.get() as f32 / 10.0
    }
    /// The temperature in Celsius
    pub fn celsius(&self) -> f32 {
        self.kelvin() - 273.15
    }
    /// Fixed-point 0.1K resolution temperature
    pub fn get(&self) -> u16 {
        self.d_kelvin.get()
    }
}
impl Display for Temperature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}K", self.d_kelvin / 10, self.d_kelvin % 10)
    }
}

/// Flags for switch and alarm change
///
/// Referred to as `DATA_FLAG` in the specification.
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct ChangeFlags(u8);
impl ChangeFlags {
    pub fn switch_change(&self) -> bool {
        self.0 & 0b0001_0000 != 0
    }
    pub fn alarm_change(&self) -> bool {
        self.0 & 0b0000_0001 != 0
    }
}
impl Display for ChangeFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let switch_change = self.switch_change();
        let alarm_change = self.alarm_change();
        write!(
            f,
            "unread switch change: {switch_change}, unread alarm change: {alarm_change}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_kelvin_formatting() {
        let temp = super::Temperature {
            d_kelvin: 1235.into(),
        };
        assert_eq!(&format!("{temp}"), "123.5K")
    }
    #[test]
    fn test_from_bytes() {
        // Millivolt
        const MILLI_VOLT: [u8; 2] = [0x0d, 0x45]; // 3397mV
        let milli_volt: &Volt<1000> = Volt::ref_from_bytes(&MILLI_VOLT).unwrap();
        assert_eq!(milli_volt.get_raw(), 3397);

        // Milliampere
        const MILLI_AMP: [u8; 2] = [0x0d, 0x45]; // 3397mA
        let milli_amp = MilliAmpere::ref_from_bytes(&MILLI_AMP).unwrap();
        assert_eq!(milli_amp.get(), 3397);

        // Milliampere-hours
        const MILLI_AMP_HOUR: [u8; 2] = [0xbf, 0x68]; // 49000mAh
        let milli_amp_hour = MilliAmpereHours::ref_from_bytes(&MILLI_AMP_HOUR).unwrap();
        assert_eq!(milli_amp_hour.get(), 49000);

        // Temperature
        const TEMPERATURE: [u8; 2] = [0x0b, 0xc3]; // 28C / 301.1K
        let temp = Temperature::ref_from_bytes(&TEMPERATURE).unwrap();
        assert_eq!(temp.get(), 3011);
    }
}
