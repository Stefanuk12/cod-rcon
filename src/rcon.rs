// Dependencies
use std::{
    net::{
        TcpStream,
        UdpSocket
    }, 
    io::Write
};

//
const FF: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
pub enum PacketType {
    CommandR = 0x02,
    Auth = 0x03,
    ResponseValue = 0x00
}
#[derive(Debug)]
pub enum RCONError {
    NotConnected,
    DisabledMode,
    NoChallengeToken,
    MalforedRead,
    RecieveAuth,
    Send,
    TCPAuth,
    ChallengeFailed,
    ConnectFailed
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
            port: 25525,
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
    pub fn connect(&mut self) -> Result<(), RCONError> {
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
            if let Err(_e) = self.send_command(&self.password, Some(PacketType::Auth), None) {
                return Err(RCONError::TCPAuth);
            }
        } else {
            // Connect to socket
            let socket_u = UdpSocket::bind(socket_address.clone());
            if let Err(_e) = socket_u {
                return Err(RCONError::ConnectFailed);
            }
            let socket = socket_u.unwrap();
            socket.connect(socket_address).unwrap();
            self.u_socket = Some(socket.try_clone().unwrap());

            // Send challenge (if enabled)
            if self.challenge.unwrap() {
                if let Err(_e) = self.send_command("challenge rcon\n", None, None) {
                    return Err(RCONError::ChallengeFailed);
                }
            }
            
            // Send some data
            self.send(&[0xff, 0xff, 0xff, 0xff, 0x00]).unwrap();
            self.auth = Some(true);
        } 

        // Success
        Ok(())
    }

    // Send data
    pub fn send(&self, data: &[u8]) -> Result<(), RCONError> {
        // Ensure we have a socket
        let tcp = self.tcp.unwrap();
        if (tcp && self.t_socket.is_none()) || (!tcp && self.u_socket.is_none()) {
            return Err(RCONError::NotConnected);
        }

        // Attempt to send
        println!("Sending command: {:02x?}", data);
        if tcp {
            if let Err(_e) = self.t_socket.as_ref().unwrap().write(data) {
                return Err(RCONError::Send);
            }
        } else if let Err(_e) = self.u_socket.as_ref().unwrap().send(data) {
            return Err(RCONError::Send);
        }

        // Success
        Ok(())
    }

    // Send a command
    pub fn send_command(&self, data: &str, command_type: Option<PacketType>, id: Option<u32>) -> Result<(), RCONError> {
        // Defaults
        let _id = id.unwrap_or(self.id.unwrap());
        let _command_type = command_type.unwrap_or(PacketType::CommandR);

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
        self.send(buf.as_slice())
    }

    // Reads UDP data
    pub fn read_udp(&mut self) -> Result<String, RCONError> {
        // Ensure we are connected
        if self.u_socket.is_none() {
            return Err(RCONError::NotConnected);
        }
        let socket = self.u_socket.as_ref().unwrap();

        // Read the data
        let mut buf = [0; 1024];
        let (_amt, _src) = socket.recv_from(&mut buf).unwrap();

        // Check for malformed
        if buf.chunks(4).next().unwrap() == FF {
            return Err(RCONError::MalforedRead);
        }

        // Check for auth
        let str_buf = String::from_utf8(buf.to_vec()).unwrap();
        let str_buf_split: Vec<_> = str_buf.split(" ").collect();
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
        Ok(str_buf[..str_buf.len() - 2].to_string())
    }

    // Reads data (tcp and udp)
    pub fn read(&mut self) -> Result<String, RCONError> {
        return self.read_udp();
    }
}