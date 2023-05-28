// Dependencies
mod rcon;
use rcon::RCON;
use crate::rcon::PacketType;

use clap::Parser;

/// Run RCON commands (via tty)
#[derive(Parser, Debug, Clone)]
#[command(name = "Call of Duty: RCON client")]
#[command(version, about, long_about = None)]
struct Args {
    /// The hostname of the RCON server
    #[arg(long, short = 'H', default_value = "127.0.0.1")]
    host: String,

    /// The port of the RCON server
    #[arg(long, short = 'P', default_value_t = 27017)]
    port: u16,

    /// The password of the RCON server
    #[arg(long, short = 'p', default_value = "password")]
    password: String,

    /// Send an optional command once connected to the RCON server
    command: Option<String>,

    /// Listens to tty then runs the input as an RCON command
    #[arg(long, short = 'O', default_value = "false")]
    tty: bool,

    /// Verbose mode (shows sending stuff)
    #[arg(long, short = 'v', default_value = "false")]
    verbose: bool
}

// Main
#[tokio::main]
async fn main() {
    // Parse cli args
    let args: Args = Args::parse();

    // Attempt to connect
    let mut rcon = RCON::default();
    rcon.host = args.host;
    rcon.port = args.port;
    rcon.password = args.password;
    let verbose = args.verbose;
    rcon.connect(verbose).await.unwrap();

    // Send the command
    let command = args.command.clone();
    if command.is_some() {
        rcon.send_command(&command.unwrap(), Some(PacketType::CommandR), None, verbose).await.unwrap();
        if verbose {
            let read_r = rcon.read(Some(true)).await;
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
                    if input.len() == 0 {
                        break;
                    }
                    if verbose { println!("received input: {}", input) };

                    if let Err(e) = rcon.send_command(&input.trim(), Some(PacketType::CommandR), None, verbose).await {
                        if verbose { println!("unable to send command - {:?}", e) }
                    };

                    // Get response (if verbose)
                    if verbose {
                        let read_r = rcon.read(Some(true)).await;
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