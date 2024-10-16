use clap::Parser;

pub mod autocompletes;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    /// The serial port of the meshtastic device. 
    #[arg(long, short)]
    pub port: Option<String>,
    /// The IP address and port of the meshtastic device. 
    #[arg(long, short)]
    pub address: Option<String>,
}
