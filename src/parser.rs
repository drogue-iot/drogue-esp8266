use nom::named;
use nom::do_parse;
use nom::tuple;
use nom::tag;
use nom::opt;
use nom::alt;
use nom::take;
use nom::char;
use nom::take_until;
use nom::character::streaming::digit1;
use nom::IResult;

use drogue_network::Ipv4Addr;

use crate::protocol::Response;
use crate::protocol::WifiConnectionFailure;
use crate::protocol::FirmwareInfo;
use crate::protocol::IpAddresses;

use crate::num::{
    ascii_to_digit,
    atoi_u8,
    atoi_usize,
};


fn parse_u8(input: &[u8]) -> IResult<&[u8], u8> {
    let (input, digits) = digit1(input)?;
    IResult::Ok((input, atoi_u8(digits).unwrap()))
}

fn parse_usize(input: &[u8]) -> IResult<&[u8], usize> {
    let (input, digits) = digit1(input)?;
    IResult::Ok((input, atoi_usize(digits).unwrap()))
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

named!(
    pub error<Response>,
    do_parse!(
        opt!(crlf) >>
        opt!(crlf) >>
        tag!("ERROR") >>
        crlf >>
        (
            Response::Error
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
    pub ready_for_data<Response>,
    do_parse!(
        tag!("> ") >>
        (
            Response::ReadyForData
        )
    )
);

named!(
    pub send_ok<Response>,
    do_parse!(
        crlf >>
        tag!("Recv ") >>
        len: parse_usize >>
        tag!(" bytes") >>
        crlf >> crlf >>
        tag!("SEND OK") >>
        crlf >>
        (
            Response::SendOk(len)
        )
    )
);

named!(
    pub data_available<Response>,
    do_parse!(
        opt!( crlf ) >>
        tag!( "+IPD,") >>
        link_id: parse_usize >>
        char!(',') >>
        len: parse_usize >>
        crlf >>
        (
            Response::DataAvailable {link_id, len }
        )
    )
);

named!(
    pub closed<Response>,
    do_parse!(
        opt!(crlf) >>
        link_id: parse_usize >>
        tag!(",CLOSED") >>
        crlf >>
        (
            Response::Closed(link_id)
        )
    )
);

named!(
    pub data_received<Response>,
    do_parse!(
        opt!(crlf) >>
        tag!("+CIPRECVDATA,") >>
        len: parse_usize >>
        char!(':') >>
        data: take!(len) >>
        crlf >>
        ok >>
        ( {
            let mut buf = [0; 128];
            for (i, b) in data.iter().enumerate() {
                //log::info!( "copy {} @ {}", *b as char, i);
                buf[i] = *b;
            }
            //log::info!("------------> onwards {:?}", buf);
            Response::DataReceived(buf, len)
        } )
    )
);


named!(
    pub parse<Response>,
    alt!(
          ok
        | error
        | firmware_info
        | wifi_connected
        | wifi_disconnect
        | wifi_connection_failure
        | got_ip
        | ip_addresses
        | connect
        | closed
        | ready_for_data
        | send_ok
        | data_available
        | data_received
    )
);
