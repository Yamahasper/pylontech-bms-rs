use std::{fs::File, path::PathBuf};

use clap::{Parser, Subcommand};

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
enum Commands {
    GetProtocolVersion,
    GetSystemParameter,
}

// $ pyloncli /dev/ttyUSB0 get_system_parameter

fn main() {
    let args = Args::parse();

    let device = File::open(args.device).unwrap();
    let device = embedded_io_adapters::std::FromStd::new(device);

    let mut bms = PylontechBms::new(device);

    match args.command {
        Commands::GetProtocolVersion => {
            println!("{}", bms.get_protocol_version().unwrap())
        }
        Commands::GetSystemParameter => println!("{}", bms.get_system_parameter().unwrap()),
    }
}
