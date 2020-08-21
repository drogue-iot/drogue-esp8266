use heapless::{
    String,
    consts::U128,
};
use drogue_network::{Ipv4Addr, SocketAddr, IpAddr};
use core::fmt::Write;

/// Type of socket connection.
#[derive(Debug)]
pub enum ConnectionType {
    TCP,
    UDP,
}

/// Commands to be sent to the ESP board.
#[derive(Debug)]
pub enum Command<'a> {
    QueryFirmwareInfo,
    JoinAp {
        ssid: &'a str,
        password: &'a str,
    },
    QueryIpAddress,
    StartConnection(usize, ConnectionType, SocketAddr),
    Send {
        link_id: usize,
        len: usize,
    },
    Receive {
        link_id: usize,
        len: usize,
    },
}

impl<'a> Command<'a> {
    pub fn as_bytes(&self) -> String<U128> {
        match self {
            Command::QueryFirmwareInfo => {
                String::from("AT+GMR")
            }
            Command::QueryIpAddress => {
                String::from("AT+CIPSTA_CUR?")
            }
            Command::JoinAp { ssid, password } => {
                let mut s = String::from("AT+CWJAP_CUR=\"");
                s.push_str(ssid).unwrap();
                s.push_str("\",\"").unwrap();
                s.push_str(password).unwrap();
                s.push_str("\"").unwrap();
                s
            }
            Command::StartConnection(link_id, connection_type, socket_addr) => {
                let mut s = String::from("AT+CIPSTART=");
                write!(s, "{},", link_id).unwrap();
                match connection_type {
                    ConnectionType::TCP => {
                        write!(s, "\"TCP\"").unwrap();
                    }
                    ConnectionType::UDP => {
                        write!(s, "\"UDP\"").unwrap();
                    }
                }
                write!(s, ",").unwrap();
                match socket_addr.ip() {
                    IpAddr::V4(ip) => {
                        let octets = ip.octets();
                        write!(s, "\"{}.{}.{}.{}\",{}",
                               octets[0],
                               octets[1],
                               octets[2],
                               octets[3],
                               socket_addr.port()
                        ).unwrap();
                    }
                    IpAddr::V6(_) => {
                        panic!("IPv6 not supported")
                    }
                }
                s as String<U128>
            }
            Command::Send { link_id, len } => {
                let mut s = String::from("AT+CIPSEND=");
                write!(s, "{},{}", link_id, len).unwrap();
                s
            }
            Command::Receive { link_id, len } => {
                let mut s = String::from("AT+CIPRECVDATA=");
                write!(s, "{},{}", link_id, len).unwrap();
                s
            }
        }
    }
}

/// Responses (including unsolicited) which may be parsed from the board.
#[derive(Debug)]
pub enum Response {
    None,
    Ok,
    Error,
    FirmwareInfo(FirmwareInfo),
    ReadyForData,
    SendOk(usize),
    DataAvailable {
        link_id: usize,
        len: usize,
    },
    DataReceived([u8; 128], usize),
    WifiConnected,
    WifiConnectionFailure(WifiConnectionFailure),
    WifiDisconnect,
    GotIp,
    IpAddresses(IpAddresses),
    Connect(usize),
    Closed(usize),
}

/// IP addresses for the board, including its own address, netmask and gateway.
#[derive(Debug)]
pub struct IpAddresses {
    pub ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub netmask: Ipv4Addr,
}

/// Version information for the ESP board.
#[derive(Debug)]
pub struct FirmwareInfo {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub build: u8,
}


/// Reasons for Wifi access-point join failures.
#[derive(Debug)]
pub enum WifiConnectionFailure {
    Timeout,
    WrongPassword,
    CannotFindTargetAp,
    ConnectionFailed,
}

impl From<u8> for WifiConnectionFailure {
    fn from(code: u8) -> Self {
        match code {
            1 => WifiConnectionFailure::Timeout,
            2 => WifiConnectionFailure::WrongPassword,
            3 => WifiConnectionFailure::CannotFindTargetAp,
            _ => WifiConnectionFailure::ConnectionFailed
        }
    }
}