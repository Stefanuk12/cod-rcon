# COD - RCON

A RCON client for Call of Duty games, primarily built for boiii. Inspired from [this](https://github.com/pushrax/node-rcon/blob/master/node-rcon.js)

Note: This does not support TCP at the moment, but could easily be implemented.

# Usage

```
An RCON client for COD that can be adapted for other games. TCP not supported!

Usage: cod-rcon.exe [OPTIONS] [COMMAND]

Arguments:
  [COMMAND]  Send an optional command once connected to the RCON server

Options:
  -H, --host <HOST>          The hostname of the RCON server [default: 127.0.0.1]
  -P, --port <PORT>          The port of the RCON server [default: 27017]
  -p, --password <PASSWORD>  The password of the RCON server [default: password]
  -O, --tty                  Listens to tty then runs the input as an RCON command
  -v, --verbose              Verbose mode (shows sending stuff)
  -h, --help                 Print help
  -V, --version              Print version
```