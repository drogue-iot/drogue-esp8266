use core::fmt;
use core::fmt::{Debug, Write};
use drogue_network::{IpAddr, Ipv4Addr, SocketAddr};
use heapless::{consts::U128, String};

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
    JoinAp { ssid: &'a str, password: &'a str },
    QueryIpAddress,
    StartConnection(usize, ConnectionType, SocketAddr),
    Send { link_id: usize, len: usize },
    Receive { link_id: usize, len: usize },
}

impl<'a> Command<'a> {
    pub fn as_bytes(&self) -> String<U128> {
        match self {
            Command::QueryFirmwareInfo => String::from("AT+GMR"),
            Command::QueryIpAddress => String::from("AT+CIPSTA_CUR?"),
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
                        write!(
                            s,
                            "\"{}.{}.{}.{}\",{}",
                            octets[0],
                            octets[1],
                            octets[2],
                            octets[3],
                            socket_addr.port()
                        )
                        .unwrap();
                    }
                    IpAddr::V6(_) => panic!("IPv6 not supported"),
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
pub enum Response {
    None,
    Ok,
    Error,
    FirmwareInfo(FirmwareInfo),
    ReadyForData,
    SendOk(usize),
    DataAvailable { link_id: usize, len: usize },
    DataReceived([u8; 128], usize),
    WifiConnected,
    WifiConnectionFailure(WifiConnectionFailure),
    WifiDisconnect,
    GotIp,
    IpAddresses(IpAddresses),
    Connect(usize),
    Closed(usize),
}

impl Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::None => f.write_str("None"),
            Response::Ok => f.write_str("Ok"),
            Response::Error => f.write_str("Error"),
            Response::FirmwareInfo(v) => f.debug_tuple("FirmwareInfo").field(v).finish(),
            Response::ReadyForData => f.write_str("ReadyForData"),
            Response::SendOk(v) => f.debug_tuple("SendOk").field(v).finish(),
            Response::DataAvailable { link_id, len } => f
                .debug_struct("DataAvailable")
                .field("link_id", link_id)
                .field("len", len)
                .finish(),
            Response::DataReceived(d, l) => dump_data("DataReceived", d, *l, f),
            Response::WifiConnected => f.write_str("WifiConnected"),
            Response::WifiConnectionFailure(v) => {
                f.debug_tuple("WifiConnectionFailure").field(v).finish()
            }
            Response::WifiDisconnect => f.write_str("WifiDisconnect"),
            Response::GotIp => f.write_str("GotIp"),
            Response::IpAddresses(v) => f.debug_tuple("IpAddresses").field(v).finish(),
            Response::Connect(v) => f.debug_tuple("Connect").field(v).finish(),
            Response::Closed(v) => f.debug_tuple("Closed").field(v).finish(),
        }
    }
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
            _ => WifiConnectionFailure::ConnectionFailed,
        }
    }
}

/// Dump some data, which is stored in a buffer with a length indicator.
///
/// The output will contain the field name, the data as string (only 7bits) and the raw bytes
/// in hex encoding.
fn dump_data(name: &str, data: &[u8], len: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let data = &data[0..len];

    f.write_str(name)?;
    f.write_char('(')?;

    f.write_fmt(format_args!("{}; '", len))?;

    for d in data {
        if *d == 0 {
            f.write_str("\\0")?;
        } else if *d <= 0x7F {
            f.write_char(*d as char)?;
        } else {
            f.write_char('\u{FFFD}')?;
        }
    }

    f.write_str("'; ")?;
    f.write_fmt(format_args!("{:X?}", data))?;
    f.write_char(')')?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use arrayvec::ArrayString;
    use core::fmt::Write;

    #[test]
    fn test_debug_no_value() {
        let mut buf = ArrayString::<[u8; 20]>::new();

        write!(&mut buf, "{:?}", Response::Ok).expect("Can't write");
        assert_eq!(&buf, "Ok");
    }

    #[test]
    fn test_debug_simple_value() {
        let mut buf = ArrayString::<[u8; 20]>::new();

        write!(&mut buf, "{:?}", Response::Connect(1)).expect("Can't write");
        assert_eq!(&buf, "Connect(1)");
    }

    #[test]
    fn test_debug_data() {
        let mut buf = ArrayString::<[u8; 256]>::new();
        let data = b"FOO\0BAR";

        let mut array = [0u8; 128];
        for (&x, p) in data.iter().zip(array.iter_mut()) {
            *p = x;
        }

        write!(&mut buf, "{:?}", Response::DataReceived(array, data.len())).expect("Can't write");
        assert_eq!(
            &buf,
            "DataReceived(7; 'FOO\\0BAR'; [46, 4F, 4F, 0, 42, 41, 52])"
        );
    }
}
