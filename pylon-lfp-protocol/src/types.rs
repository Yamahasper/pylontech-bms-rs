//! Datatypes and units

use core::fmt::Display;

/// Millivolt
#[derive(Debug)]
pub struct MilliVolt(i16);
impl Display for MilliVolt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}mV", self.0)
    }
}
/// Milliampere
#[derive(Debug)]
pub struct MilliAmpere(i16);
impl Display for MilliAmpere {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}mA", self.0)
    }
}
/// Milliampere-hours
#[derive(Debug)]
pub struct MilliAmpereHours(u16);
impl Display for MilliAmpereHours {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}mAh", self.0)
    }
}
/// Temperature
#[derive(Debug)]
pub struct Temperature {
    /// dK (dezi Kelvin) (0.1K)
    d_kelvin: u16,
}
impl Display for Temperature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}K", self.d_kelvin / 10, self.d_kelvin % 10)
    }
}

/// Flags for switch and alarm change
///
/// Referred to as `DATA_FLAG` in the specification.
#[derive(Debug)]
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
    #[test]
    fn test_kelvin_formatting() {
        let temp = super::Temperature { d_kelvin: 1235 };
        assert_eq!(&format!("{temp}"), "123.5K")
    }
}
