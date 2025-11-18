use core::fmt::Display;

use crate::types::{Ampere, Temperature, Volt, exponents::MILLI};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct SystemParameter<
    const CELL_VOLTAGE_EXP: i8 = MILLI,
    const TOTAL_VOLTAGE_EXP: i8 = MILLI,
    const CURRENT_EXP: i8 = MILLI,
    const TEMP_EXP: i8 = MILLI,
> {
    pub unit_cell_voltage: Volt<CELL_VOLTAGE_EXP>,
    pub unit_cell_low_voltage_threshold: Volt<CELL_VOLTAGE_EXP>,
    /// Under voltage protection threshold
    pub unit_cell_under_voltage_threshold: Volt<CELL_VOLTAGE_EXP>,
    pub charge_upper_limit_temp: Temperature<TEMP_EXP>,
    pub charge_lower_limit_temp: Temperature<TEMP_EXP>,
    pub charge_lower_limit_current: Ampere<CURRENT_EXP>,
    pub upper_limit_total_voltage: Volt<TOTAL_VOLTAGE_EXP>,
    pub lower_limit_total_voltage: Volt<TOTAL_VOLTAGE_EXP>,
    pub under_voltage_of_total_voltage: Volt<TOTAL_VOLTAGE_EXP>,
    pub discharge_upper_limit_temp: Temperature<TEMP_EXP>,
    pub discharge_lower_limit_temp: Temperature<TEMP_EXP>,
    pub discharge_lower_limit_current: Ampere<CURRENT_EXP>,
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
