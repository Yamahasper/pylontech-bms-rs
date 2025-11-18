use std::{borrow::Cow, path::PathBuf, time::Duration};

use clap::{Parser, Subcommand, ValueEnum};

use embedded_io::{Read, Write};
use embedded_io_adapters::std::FromStd;
use pylon_lfp_protocol::{PylontechBms, commands::PackData, types::exponents::*};

/// A Command Line tool to interact with batteries implementing the Pylontech RS232 protocol
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Serial device to use
    device: PathBuf,

    /// Battery pack address
    #[arg(short, long, default_value_t = 1)]
    address: u8,

    /// Baud rate
    #[arg(short, long, default_value_t = 9600)]
    baud: u32,

    /// Timeout in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    timeout: u64,

    /// Battery pack type (omit for specification default)
    #[arg(short, long)]
    flavor: Option<Flavor>,

    /// Command
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Commands {
    /// Query the protocol version of a pack
    GetProtocolVersion,
    /// Get system parameters of a pack
    GetSystemParameter,
    /// Get live measurements of one or more packs
    GetAnalogValue {
        /// Battery pack to query, all packs are queried if not specified
        #[arg(short, long)]
        pack_address: Option<u8>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Flavor {
    /// Superpack branded packs
    Superpack,
}

fn main() {
    let args = Args::parse();

    let port = serialport::new(Cow::from(args.device.to_str().unwrap()), args.baud)
        .timeout(Duration::from_millis(args.timeout))
        .open()
        .unwrap();

    let device = FromStd::new(port);

    let mut bms = PylontechBms::new(device);

    match args.command {
        Commands::GetProtocolVersion => {
            println!("{}", bms.get_protocol_version().unwrap())
        }
        Commands::GetSystemParameter => println!("{}", bms.get_system_parameter().unwrap()),
        Commands::GetAnalogValue { pack_address } => {
            get_and_print_analog_values(&mut bms, pack_address, args.flavor)
        }
    }
}

fn get_and_print_analog_values<T: Read + Write>(
    bms: &mut PylontechBms<T>,
    adr: Option<u8>,
    flavor: Option<Flavor>,
) {
    let mut buf = [0; pylon_lfp_protocol::MAX_UNENCODED_PAYLOAD_LEN];
    let measurements = bms.get_analog_value(adr.unwrap_or(0xFF), &mut buf).unwrap();
    if measurements.flags.switch_change() {
        println!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
        println!("!! Unread switch change !!");
        println!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
    }
    if measurements.flags.alarm_change() {
        println!("!!!!!!!!!!!!!!!!!!!!!!!!!");
        println!("!! Unread alarm change !!");
        println!("!!!!!!!!!!!!!!!!!!!!!!!!!");
    }
    for i in 0..measurements.get_pack_count() {
        println!("=========");
        println!("Pack {i}:");
        println!("=========");
        match flavor {
            Some(Flavor::Superpack) => {
                let pack: PackData<'_, MILLI, CENTI, CENTI, CENTI> =
                    measurements.get_pack(i).unwrap();
                print_pack(pack);
            }
            None => {
                let pack: PackData<'_> = measurements.get_pack(i).unwrap();
                print_pack(pack);
            }
        }
    }
}

fn print_pack<
    const CELL_VOLT_EXP: i8,
    const TOTAL_VOLT_EXP: i8,
    const CURRENT_EXP: i8,
    const AMP_HOUR_EXP: i8,
>(
    pack: PackData<'_, CELL_VOLT_EXP, TOTAL_VOLT_EXP, CURRENT_EXP, AMP_HOUR_EXP>,
) {
    for (n, v) in pack.cell_voltages.iter().enumerate() {
        println!("Voltage {n}: {v}");
    }
    for (n, t) in pack.temperatures.iter().enumerate() {
        println!("Temp {n}: {:#}", t);
    }
    println!("Current: {}", pack.pack_current);
    println!("Total Voltage: {}", pack.pack_voltage);
    println!("Remaining capacity: {}", pack.pack_remaining);
    println!("Total capacity: {}", pack.total_capacity);
    println!("Cell cycles: {}", pack.cell_cycles);
}
