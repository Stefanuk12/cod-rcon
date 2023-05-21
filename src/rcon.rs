// Dependencies
use std::{
    net::{
        TcpStream,
        UdpSocket
    }, 
    io::{Write, ErrorKind}
};

// Constants
const FF: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

// Used for TCP
pub enum PacketType {
    CommandR = 0x02,
    Auth = 0x03,
    ResponseValue = 0x00
}

// Errors
#[derive(Debug)]
pub enum RCONError {
    NotConnected,
    DisabledMode,
    NoChallengeToken,
    MalforedRead,
    RecieveAuth,
    TCPAuth,
    ChallengeFailed,
    ConnectFailed,
    ErrorKind(ErrorKind)
}

// Main struct
pub struct RCON {
    pub host: String,
    pub port: u16,
    pub password: String,

    pub id: Option<u32>,
    pub tcp: Option<bool>,
    pub challenge: Option<bool>,
    pub challenge_token: Option<String>,
    pub auth: Option<bool>,

    pub t_socket: Option<TcpStream>,
    pub u_socket: Option<UdpSocket>
}

// Funcs
impl RCON {
    // Default data
    pub fn default() -> RCON {
        Self {
            host: String::from(""),
            port: 27017,
            password: String::from(""),

            id: Some(0x0012D4A6),
            tcp: Some(false),
            challenge: Some(false),
            challenge_token: None,
            auth: Some(false),
            t_socket: None,
            u_socket: None,
        }
    }

    // Initialises the sockets
    pub fn connect(&mut self, verbose: Option<bool>) -> Result<(), RCONError> {
        // Initialise defaults
        self.id = Some(self.id.unwrap_or(0x0012D4A6));
        self.tcp = Some(self.tcp.unwrap_or(false));
        self.challenge = Some(self.challenge.unwrap_or(false));
        self.auth = Some(false);

        // Socket data
        let socket_address = format!("{}:{}", self.host, self.port);
        let tcp = self.tcp.unwrap();

        //
        if tcp {
            // Connect to the stream
            let socket = TcpStream::connect(socket_address);
            if let Err(_e) = socket {
                return Err(RCONError::ConnectFailed);
            }
            self.t_socket = Some(socket.unwrap());

            // Send auth
            if let Err(_e) = self.send_command(&self.password, Some(PacketType::Auth), None, verbose) {
                return Err(RCONError::TCPAuth);
            }
        } else {
            // Bind to socket (use random port on localhost)
            let socket_u = UdpSocket::bind("0.0.0.0:0");
            if let Err(_e) = socket_u {
                return Err(RCONError::ConnectFailed);
            }

            // Connect to the socket at the correct port and config
            let socket = socket_u.unwrap();
            if let Err(_e) = socket.connect(socket_address) {
                return Err(RCONError::ConnectFailed);
            }
            socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            self.u_socket = Some(socket.try_clone().unwrap());

            // Send challenge (if enabled)
            if self.challenge.unwrap() {
                if let Err(_e) = self.send_command("challenge rcon\n", None, None, verbose) {
                    return Err(RCONError::ChallengeFailed);
                }
            }
            
            // Send some data
            self.send(&[0xff, 0xff, 0xff, 0xff, 0x00], verbose).unwrap();
            self.auth = Some(true);
        } 

        // Success
        Ok(())
    }

    // Send data
    pub fn send(&self, data: &[u8], verbose: Option<bool>) -> Result<(), RCONError> {
        // Ensure we have a socket
        let tcp = self.tcp.unwrap();
        if (tcp && self.t_socket.is_none()) || (!tcp && self.u_socket.is_none()) {
            return Err(RCONError::NotConnected);
        }

        // Attempt to send
        let verbose = verbose.unwrap_or(false);
        if verbose { println!("Sending command: {:02x?}", data); }
        if tcp {
            if let Err(e) = self.t_socket.as_ref().unwrap().write(data) {
                return Err(RCONError::ErrorKind(e.kind()));
            }
        } else if let Err(e) = self.u_socket.as_ref().unwrap().send(data) {
            return Err(RCONError::ErrorKind(e.kind()));
        }

        // Success
        Ok(())
    }

    // Send a command (UDP)
    pub fn send_command_udp(&self, data: &str, verbose: Option<bool>) -> Result<(), RCONError> {
        // Check for TCP (disabled)
        if self.tcp.unwrap() {
            return Err(RCONError::DisabledMode);
        }

        // Start to form the payload
        let mut payload = String::from("rcon ");

        // Check for challenge
        if self.challenge.unwrap() {
            if self.challenge_token.is_none() {
                return Err(RCONError::NoChallengeToken);
            } else {
                let challenge = self.challenge_token.as_ref().unwrap().clone();
                payload.push_str(&(challenge + "\n"));
            }
        }

        // Add the password and data
        payload.push_str(&(self.password.clone() + " "));
        payload.push_str(&(data.to_owned() + "\n"));

        // Construct the buffer
        let buf = [FF.to_vec(), payload.as_bytes().to_vec()].concat();

        // Send the command
        self.send(buf.as_slice(), verbose)
    }

    // Sends a command (TCP)
    pub fn send_command_tcp(&self, data: &str, command_type: Option<PacketType>, id: Option<u32>, verbose: Option<bool>) -> Result<(), RCONError> {
        // Defaults
        let _id = id.unwrap_or(self.id.unwrap());
        let _command_type = command_type.unwrap_or(PacketType::CommandR);

        // Construct the buffer
        let buf = [];

        // Send the command
        self.send(buf.as_slice(), verbose)
    }

    // Sends a command
    pub fn send_command(&self, data: &str, command_type: Option<PacketType>, id: Option<u32>, verbose: Option<bool>) -> Result<(), RCONError> {
        if self.tcp.unwrap() {
            return self.send_command_tcp(data, command_type, id, verbose);
        } else {
            return self.send_command_udp(data, verbose);
        }
    }

    // Reads TCP data
    pub fn read_tcp(&mut self, verbose: Option<bool>) -> Result<String, RCONError> {
        Ok(String::from(""))
    }

    // Reads UDP data
    pub fn read_udp(&mut self, verbose: Option<bool>) -> Result<String, RCONError> {
        // Ensure we are connected
        if self.u_socket.is_none() {
            return Err(RCONError::NotConnected);
        }
        let socket = self.u_socket.as_ref().unwrap();

        // Read the data
        let mut buf = vec![0];
        if let Err(e) = socket.recv_from(&mut buf) {
            return Err(RCONError::ErrorKind(e.kind()));
        }
        if verbose.unwrap_or(false) { println!("Received (bytes): {:02x?}", buf); }

        // Check for malformed
        if buf.chunks(4).next().unwrap() != FF {
            return Err(RCONError::MalforedRead);
        }

        // Convert to a string
        let str_buf = buf[4..].to_vec();
        let str_utf8 = String::from_utf8(str_buf).unwrap();
        let str_buf_split: Vec<_> = str_utf8.split(" ").collect();

        // Challenge check
        if str_buf_split.len() == 3 && str_buf_split[0] == "challenge" && str_buf_split[1] == "rcon" {
            // Set auth data
            self.challenge_token = Some(
                str_buf_split[2].trim().to_owned()
            );
            self.auth = Some(true);

            // Return
            return Err(RCONError::RecieveAuth);
        }

        // Return the data
        Ok(str_utf8[..str_utf8.len() - 2].to_string())
    }

    // Reads data (tcp and udp)
    pub fn read(&mut self, verbose: Option<bool>) -> Result<String, RCONError> {
        if self.tcp.unwrap() {
            return self.read_tcp(verbose);
        } else {
            return self.read_udp(verbose);
        }
    }
}