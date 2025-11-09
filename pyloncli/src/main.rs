use std::{fs::File, path::PathBuf};

use clap::{Parser, Subcommand};

use embedded_io_adapters::std::FromStd;
use pylon_lfp_protocol::PylontechBms;

/// A Command Line tool to interact with batteries implementing the Pylontech RS232 protocol
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Serial device to use
    device: PathBuf,

    /// Battery pack address
    #[arg(short, long, default_value_t = 1)]
    address: u8,

    /// Command
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Commands {
    GetProtocolVersion,
    GetSystemParameter,
    GetAnalogValue,
}

// $ pyloncli /dev/ttyUSB0 get_system_parameter

fn main() {
    let args = Args::parse();

    let mut options = File::options();
    options.read(true).write(true).append(false).create(false);
    let device = options.open(args.device).unwrap();
    let device = embedded_io_adapters::std::FromStd::new(device);

    let mut bms = PylontechBms::new(device);

    match args.command {
        Commands::GetProtocolVersion => {
            println!("{}", bms.get_protocol_version().unwrap())
        }
        Commands::GetSystemParameter => println!("{}", bms.get_system_parameter().unwrap()),
        Commands::GetAnalogValue => get_and_print_analog_values(&mut bms),
    }
}

fn get_and_print_analog_values(bms: &mut PylontechBms<FromStd<File>>) {
    let mut buf = [0; pylon_lfp_protocol::MAX_UNENCODED_PAYLOAD_LEN];
    let measurements = bms.get_analog_value(0xFF, &mut buf).unwrap();
    println!("{:?}", measurements.flags);
    for i in 0..measurements.get_pack_count() {
        println!("Pack {i}:");
        println!("{:#?}", measurements.get_pack(i));
    }
}
