// Dependencies
mod rcon;
use rcon::RCON;
use crate::rcon::PacketType;

use clap::Parser;

// Commandline args
#[derive(Parser, Debug, Clone)]
#[command(name = "Call of Duty: RCON client")]
#[command(version, about, long_about = None)]
struct Args {
    // Host to connect to
    #[arg(long, short = 'H', default_value = "127.0.0.1")]
    host: String,

    // Port
    #[arg(long, short = 'P', default_value_t = 27017)]
    port: u16,

    // Password
    #[arg(long, short = 'p', default_value = "password")]
    password: String,

    // Send an optional command
    command: Option<String>,

    // Listens to tty
    #[arg(long, short = 'O')]
    tty: bool,

    // Verbose mode (shows sending stuff)
    #[arg(long, short = 'v')]
    verbose: Option<bool>
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
    let verbose = args.verbose.clone();
    rcon.connect(verbose).unwrap();

    // Send the command
    let command = args.command.clone();
    if command.is_some() {
        rcon.send_command(&command.unwrap(), Some(PacketType::CommandR), None, verbose).unwrap();
        if verbose.unwrap_or(false) {
            let read_r = rcon.read(Some(true));
            if let Ok(resp) = read_r {
                println!("{}", resp);
            } else {
                println!("unable to read: {:?}", read_r.unwrap_err());
            }
        }
    }

    // Listen for tty commands and then route to rcon
    if args.tty {
        // Constantly grab input
        loop {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_n) => {
                    // Send the command
                    input = input.trim().to_owned();
                    if let Err(e) = rcon.send_command(&input.trim(), Some(PacketType::CommandR), None, verbose) {
                        println!("unable to send command - {:?}", e)
                    };

                    // Get response (if verbose)
                    if verbose.unwrap_or(false) {
                        let read_r = rcon.read(Some(true));
                        if let Ok(resp) = read_r {
                            println!("{}", resp);
                        } else {
                            println!("unable to read: {:?}", read_r.unwrap_err());
                        }
                    }
                }
                Err(error) => println!("error: {error}"),
            }
        }
    }
}