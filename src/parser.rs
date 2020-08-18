
use nom::named;
use nom::do_parse;
use nom::tuple;
use nom::tag;
use nom::opt;
use nom::alt;
use nom::char;
use nom::take_until;
use nom::character::streaming::digit1;
use nom::IResult;

use embedded_nal::Ipv4Addr;

use crate::protocol::Response;
use crate::protocol::WifiConnectionFailure;
use crate::protocol::FirmwareInfo;
use crate::protocol::IpAddresses;

use crate::num::{
    ascii_to_digit,
    atoi_u8,
};


fn parse_u8(input: &[u8]) -> IResult<&[u8], u8> {
    let (input, digits) = digit1(input)?;
    IResult::Ok((input, atoi_u8(digits).unwrap()))
}

#[rustfmt::skip]
named!(
    crlf,
    tag!("\r\n")
);

#[rustfmt::skip]
named!(
    pub ok<Response>,
    do_parse!(
        tuple!(
            opt!(crlf),
            opt!(crlf),
            tag!("OK"),
            crlf
        ) >>
        (
            Response::Ok
        )
    )
);

#[rustfmt::skip]
named!(
    pub wifi_connected<Response>,
    do_parse!(
        tuple!(
            tag!("WIFI CONNECTED"),
            crlf
        ) >>
        (
            Response::WifiConnected
        )
    )
);

#[rustfmt::skip]
named!(
    pub wifi_disconnect<Response>,
    do_parse!(
        tuple!(
            tag!("WIFI DISCONNECT"),
            crlf
        ) >>
        (
            Response::WifiDisconnect
        )
    )
);

#[rustfmt::skip]
named!(
    pub got_ip<Response>,
    do_parse!(
        tuple!(
            tag!("WIFI GOT IP"),
            crlf
        ) >>
        (
            Response::GotIp
        )
    )
);

named!(
    pub wifi_connection_failure<Response>,
    do_parse!(
        tag!("+CWJAP:") >>
        code: parse_u8 >>
        crlf >>
        crlf >>
        tag!("FAIL") >>
        crlf >>
        (
            Response::WifiConnectionFailure(WifiConnectionFailure::from(code))
        )
    )
);

#[rustfmt::skip]
named!(
    pub firmware_info<Response>,
    do_parse!(
        tag!("AT version:") >>
        major: parse_u8 >>
        tag!(".") >>
        minor: parse_u8 >>
        tag!(".") >>
        patch: parse_u8 >>
        tag!(".") >>
        build: parse_u8 >>
        take_until!("OK") >>
        ok >>
        (
            Response::FirmwareInfo(FirmwareInfo{major, minor, patch, build})
        )
    )
);

#[rustfmt::skip]
named!(
    ip_addr<Ipv4Addr>,
    do_parse!(
        a: parse_u8 >>
        char!('.') >>
        b: parse_u8 >>
        char!('.') >>
        c: parse_u8 >>
        char!('.') >>
        d: parse_u8 >>
        (
            Ipv4Addr::new(a, b, c, d)
        )
    )
);


#[rustfmt::skip]
named!(
    pub ip_addresses<Response>,
    do_parse!(
        tag!("+CIPSTA_CUR:ip:\"") >>
        ip: ip_addr >>
        tag!("\"") >>
        crlf >>
        tag!("+CIPSTA_CUR:gateway:\"") >>
        gateway: ip_addr >>
        tag!("\"") >>
        crlf >>
        tag!("+CIPSTA_CUR:netmask:\"") >>
        netmask: ip_addr >>
        tag!("\"") >>
        crlf >>
        crlf >>
        ok >>
        (
            Response::IpAddresses(
                IpAddresses {
                    ip,
                    gateway,
                    netmask,
                }
            )
        )
    )
);

#[rustfmt::skip]
named!(
    pub connect<Response>,
    do_parse!(
        link_id: parse_u8 >>
        tag!(",CONNECT") >>
        crlf >>
        ok >>
        (
            Response::Connect(link_id as usize)
        )
    )
);




named!(
    pub parse<Response>,
    alt!(
          ok
        | firmware_info
        | wifi_connected
        | wifi_disconnect
        | wifi_connection_failure
        | got_ip
        | ip_addresses
        | connect
    )
);
