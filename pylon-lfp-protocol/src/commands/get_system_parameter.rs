use core::fmt::Display;

use crate::types::{MilliAmpere, MilliVolt, Temperature};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct SystemParameter {
    pub unit_cell_voltage: MilliVolt,
    pub unit_cell_low_voltage_threshold: MilliVolt,
    /// Under voltage protection threshold
    pub unit_cell_under_voltage_threshold: MilliVolt,
    pub charge_upper_limit_temp: Temperature,
    pub charge_lower_limit_temp: Temperature,
    pub charge_lower_limit_current: MilliAmpere,
    pub upper_limit_total_voltage: MilliVolt,
    pub lower_limit_total_voltage: MilliVolt,
    pub under_voltage_of_total_voltage: MilliVolt,
    pub discharge_upper_limit_temp: Temperature,
    pub discharge_lower_limit_temp: Temperature,
    pub discharge_lower_limit_current: MilliAmpere,
}

impl Display for SystemParameter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Cell voltage: {}", self.unit_cell_voltage)?;
        writeln!(
            f,
            "Cell low voltage threshold: {}",
            self.unit_cell_low_voltage_threshold
        )?;
        writeln!(
            f,
            "Cell under voltage voltage threshold: {}",
            self.unit_cell_under_voltage_threshold
        )?;
        writeln!(
            f,
            "Charge current upper limit temperature: {}",
            self.charge_upper_limit_temp
        )?;
        writeln!(
            f,
            "Charge current lower limit temperature: {}",
            self.charge_lower_limit_temp
        )?;
        writeln!(
            f,
            "Charge current lower limit current: {}",
            self.charge_lower_limit_current
        )?;
        writeln!(
            f,
            "Upper limit total voltage: {}",
            self.upper_limit_total_voltage
        )?;
        writeln!(
            f,
            "Lower limit total voltage: {}",
            self.lower_limit_total_voltage
        )?;
        writeln!(
            f,
            "Total voltage under voltage threshold: {}",
            self.under_voltage_of_total_voltage
        )?;
        writeln!(
            f,
            "Discharge upper limit temperature: {}",
            self.discharge_upper_limit_temp
        )?;
        writeln!(
            f,
            "Discharge lower limit temperature: {}",
            self.discharge_lower_limit_temp
        )?;
        writeln!(
            f,
            "Discharge lower limit current: {}",
            self.discharge_lower_limit_current
        )
    }
}
