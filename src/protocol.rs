use heapless::{
    String,
    consts::{
        U64,
        U128,
    },
};
use crate::protocol::Response::{WifiDisconnect, WifiConnected};
use embedded_nal::{Ipv4Addr, SocketAddr, IpAddr};
use heapless::i::Vec;
use core::fmt::Write;

#[derive(Debug)]
pub enum ConnectionType {
    TCP,
    UDP,
}

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
                s.push_str(ssid);
                s.push_str("\",\"");
                s.push_str(password);
                s.push_str("\"");
                s
            }
            Command::StartConnection(link_id, connection_type, socket_addr) => {
                let mut s = String::from("AT+CIPSTART=");
                write!(s, "{},", link_id);
                //let mut s: heapless::Vec<u8, U128> = heapless::Vec::new();
                //let s = s.into_bytes();
                match connection_type {
                    ConnectionType::TCP => {
                        write!(s, "\"TCP\"");
                    }
                    ConnectionType::UDP => {
                        write!(s, "\"UDP\"");
                    }
                }
                write!(s, ",");
                match socket_addr.ip() {
                    IpAddr::V4(ip) => {
                        let octets = ip.octets();
                        write!(s, "\"{}.{}.{}.{}\",{}",
                               octets[0],
                               octets[1],
                               octets[2],
                               octets[3],
                               socket_addr.port()
                        );
                    }
                    IpAddr::V6(_) => {
                        panic!("IPv6 not supported")
                    }
                }
                s as String<U128>
            }
            Command::Send { link_id, len } => {
                let mut s = String::from("AT+CIPSEND=");
                write!(s, "{},{}", link_id, len);
                s
            }
            Command::Receive { link_id, len } => {
                let mut s = String::from("AT+CIPRECVDATA=");
                write!(s, "{},{}", link_id, len);
                s
            }
        }
    }
}

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

#[derive(Debug)]
pub struct IpAddresses {
    pub ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub netmask: Ipv4Addr,
}

#[derive(Debug)]
pub struct FirmwareInfo {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub build: u8,
}

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