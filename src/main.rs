// Dependencies
mod rcon;
use clap::Parser;
use rcon::RCON;

// Commandline args
#[derive(Parser, Debug, Clone)]
#[command(name = "Call of Duty: RCON client")]
#[command(version, about, long_about = None)]
struct Args {
    // Host to connect to
    host: String,

    // Port
    port: u16,

    // Password
    password: String,

    // Command
    command: String,
}

// Main
fn main() {
    // Parse cli args
    let args: Args = Args::parse();

    // Attempt to connect
    let mut rcon = RCON::default();
    rcon.host = args.host;
    rcon.port = args.port;
    rcon.password = args.password;
    rcon.connect().unwrap();

    // Send a test message
    rcon.send_command(&args.command, None, None).unwrap();
}
