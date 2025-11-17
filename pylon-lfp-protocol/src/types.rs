//! Datatypes and units
//!
//! ## Binary representation of measurements
//!
//! Many of the units specified in this module have an
//! associated _const generic_ to specify an exponent.
//! This is used to specify the _metric prefix_ of the
//! binary representation.
//!
//! The PYLON Tech specification defines a binary representation
//! for every type, which is used as default for the protocol data structures.
//! The reason the _metric prefix_ (exponent) is exposed to the user becomes
//! apparent when considering the ranges for the stored values.
//!
//! _Example:_ The specification defines pack capacity in mAh
//! stored as (unsigned) 16-bit, which limits the range from 0 mAh to
//! 65536 mAh or roughly 65 Ah.
//! To counteract this, a different exponent can be used for measurements
//! that couldn't be represented otherwise.
//!
//! Some battery packs seem to violate the specification for this reason.
//! This has been observed on a 60V 100Ah "_Superpack_" branded battery pack.

use core::fmt::Display;
use zerocopy::byteorder::big_endian;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

use exponents::*;

/// Metric prefixes used with type signatures
pub mod exponents {
    pub const NANO: i8 = -6;
    pub const MICRO: i8 = -6;
    pub const MILLI: i8 = -3;
    pub const CENTI: i8 = -2;
    pub const DECI: i8 = -1;
    pub const DECA: i8 = 1;
    pub const HECTO: i8 = 2;
    pub const KILO: i8 = 3;
    pub const MEGA: i8 = 6;
    pub(crate) const fn number(exp: i8) -> f32 {
        match exp {
            x if x == NANO => 0.000_000_001,
            x if x == MICRO => 0.000_001,
            x if x == MILLI => 0.001,
            x if x == CENTI => 0.01,
            x if x == DECI => 0.1,
            x if x == DECA => 10.,
            x if x == HECTO => 100.,
            x if x == KILO => 1_000.,
            x if x == MEGA => 1_000_000.,
            _ => panic!("Exponent not supported"),
        }
    }
}

/// Type alias for a voltage stored in Millivolt
pub type MilliVolt = Volt<MILLI>;

/// Voltage
///
/// Holds a scaled voltage in Volt.
/// `EXP` is the metric prefix the voltage is stored in (e.g. a voltage stored in mV has a exponent of `-3`).
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct Volt<const EXP: i8>(big_endian::U16);
impl<const EXP: i8> Display for Volt<EXP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if EXP == NANO {
            write!(f, "{} nV", self.0)
        } else if EXP == MILLI {
            write!(f, "{} mV", self.0)
        } else if EXP == KILO {
            write!(f, "{} kV", self.0)
        } else {
            write!(f, "{:.2} V", self.get_volt())
        }
    }
}
impl<const EXP: i8> Volt<EXP> {
    /// Get the raw stored value
    ///
    /// The protocol specifies voltage as 16-bit.
    /// Since the examples contain values greater than `32_768`,
    /// a unsigned integer is needed (eliminating the possibility
    /// to store/transmit negative voltages).
    pub fn get_raw(&self) -> u16 {
        self.0.get()
    }
    /// Floating-point voltage in Volt
    pub fn get_volt(&self) -> f32 {
        self.get_raw() as f32 * number(EXP)
    }
}

/// Type alias for a current stored in Milliampere
pub type MilliAmpere = Ampere<MILLI>;

/// Current
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct Ampere<const EXP: i8>(big_endian::I16);
impl<const EXP: i8> Display for Ampere<EXP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if EXP == NANO {
            write!(f, "{} nA", self.0)
        } else if EXP == MILLI {
            write!(f, "{} mA", self.0)
        } else if EXP == KILO {
            write!(f, "{} kA", self.0)
        } else {
            write!(f, "{:.2} A", self.get_ampere())
        }
    }
}
impl<const EXP: i8> Ampere<EXP> {
    /// Get the raw stored value
    pub fn get_raw(&self) -> i16 {
        self.0.get()
    }
    /// Floating-point current in Ampere
    pub fn get_ampere(&self) -> f32 {
        self.get_raw() as f32 * number(EXP)
    }
}

/// Type alias for a charge stored in Milliampere-hours
pub type MilliAmpereHours = AmpereHours<MILLI>;

/// Electric charge
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct AmpereHours<const EXP: i8>(big_endian::U16);
impl<const EXP: i8> Display for AmpereHours<EXP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if EXP == NANO {
            write!(f, "{} nAh", self.0)
        } else if EXP == MILLI {
            write!(f, "{} mAh", self.0)
        } else if EXP == KILO {
            write!(f, "{} kAh", self.0)
        } else {
            write!(f, "{:.2} Ah", self.get_ampere_hours())
        }
    }
}
impl<const EXP: i8> AmpereHours<EXP> {
    /// Get the raw stored value
    pub fn get_raw(&self) -> u16 {
        self.0.get()
    }
    /// Floating-point charge in Ampere-hours
    pub fn get_ampere_hours(&self) -> f32 {
        self.get_raw() as f32 * number(EXP)
    }
}

/// Temperature
///
/// Temperature is stored in Kelvin with a specified metric prefix (exponent).
/// The Temperature can be converted to a floating-point value in Kelvin or degree Celsius.
///
/// The [Display] (`{}`) formatting displays the temperature in Kelvin with a precision of `1` by default.
/// This can be changed by specifying the precision (e.g. `{:.2}`).
/// Using the _alternate form_ (`{:#}`) displays the temperature in degree Celsius (`°C`).
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(transparent)]
pub struct Temperature<const EXP: i8>(big_endian::U16);
impl<const EXP: i8> Temperature<EXP> {
    /// The temperature in Kelvin
    pub fn kelvin(&self) -> f32 {
        self.0.get() as f32 * number(EXP)
    }
    /// The temperature in degree Celsius
    pub fn celsius(&self) -> f32 {
        self.kelvin() - 273.15
    }
    /// Get the raw stored value
    pub fn get_raw(&self) -> u16 {
        self.0.get()
    }
}
/// Display the temperature in Kelvin; alternate form displays in degree Celsius
impl<const EXP: i8> Display for Temperature<EXP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            if let Some(precision) = f.precision() {
                write!(f, "{1:.*} °C", precision, self.celsius())
            } else {
                write!(f, "{:.1} °C", self.celsius())
            }
        } else if let Some(precision) = f.precision() {
            write!(f, "{1:.*} K", precision, self.kelvin())
        } else {
            write!(f, "{:.1} K", self.kelvin())
        }
    }
}

/// Temperature representation defined by the specification
pub type DeciKelvin = Temperature<DECI>;

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
    fn test_temp_formatting() {
        let temp: Temperature<{ exponents::DECI }> = super::Temperature(1235.into());
        assert_eq!(&format!("{temp}"), "123.5 K");
        assert_eq!(&format!("{temp:.2}"), "123.50 K");
        assert_eq!(&format!("{temp:#}"), "-149.6 °C");
        assert_eq!(&format!("{temp:#.2}"), "-149.65 °C");
    }
    #[test]
    fn test_from_bytes() {
        // Millivolt
        const MILLI_VOLT: [u8; 2] = [0x0d, 0x45]; // 3397mV
        let milli_volt: &MilliVolt = Volt::ref_from_bytes(&MILLI_VOLT).unwrap();
        assert_eq!(milli_volt.get_raw(), 3397);

        // Milliampere
        const MILLI_AMP: [u8; 2] = [0x0d, 0x45]; // 3397mA
        let milli_amp: &MilliAmpere = Ampere::ref_from_bytes(&MILLI_AMP).unwrap();
        assert_eq!(milli_amp.get_raw(), 3397);

        // Milliampere-hours
        const MILLI_AMP_HOUR: [u8; 2] = [0xbf, 0x68]; // 49000mAh
        let milli_amp_hour: &MilliAmpereHours =
            AmpereHours::ref_from_bytes(&MILLI_AMP_HOUR).unwrap();
        assert_eq!(milli_amp_hour.get_raw(), 49000);

        // Temperature
        const TEMPERATURE: [u8; 2] = [0x0b, 0xc3]; // 28C / 301.1K
        let temp: &DeciKelvin = Temperature::ref_from_bytes(&TEMPERATURE).unwrap();
        assert_eq!(temp.get_raw(), 3011);
    }
    #[test]
    fn format_volt() {
        use exponents::*;
        // Millivolt
        const MILLI_VOLT: [u8; 2] = [0x0d, 0x45]; // 3397mV
        let milli_volt: &MilliVolt = Volt::ref_from_bytes(&MILLI_VOLT).unwrap();
        assert_eq!(milli_volt.get_raw(), 3397);
        assert_eq!(format!("{milli_volt}"), "3397 mV");

        // Nanovolt
        let nano_volt: Volt<NANO> = Volt(123.into());
        assert_eq!(format!("{nano_volt}"), "123 nV");

        // Hundredth of Volt
        let hundredth_volt: Volt<DECI> = Volt(123.into());
        assert_eq!(format!("{hundredth_volt}"), "12.30 V");
    }
}
