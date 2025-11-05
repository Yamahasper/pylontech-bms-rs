//! Datatypes and units

use core::fmt::Display;

/// Millivolt
#[derive(Debug)]
pub struct MilliVolt(u16);
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
/// Temperature in dK (0.1K)
#[derive(Debug)]
pub struct Temperature(u16);
impl Display for Temperature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}dK", self.0)
    }
}
