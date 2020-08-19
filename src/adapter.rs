use embedded_hal::{
    digital::v2::OutputPin,
    serial::Read,
    serial::Write,
};

use crate::Error;
use crate::protocol::{Command, Response, WifiConnectionFailure, FirmwareInfo, IpAddresses, ConnectionType};

use heapless::{i, spsc::{
    Queue,
    Consumer,
    Producer,
}, consts::{
    U2,
    U2048,
}, Vec, ArrayLength};

use log::info;

use crate::ingress::Ingress;
use crate::network::{NetworkStack, Socket, Sockets};
use embedded_nal::SocketAddr;

type Initialized<'a, Tx, Rx> = (
    Adapter<'a, Tx>,
    Ingress<'a, Rx>
);

pub fn initialize<'a, Tx, Rx, EnablePin, ResetPin>(
    mut tx: Tx,
    mut rx: Rx,
    enable_pin: &mut EnablePin,
    reset_pin: &mut ResetPin,
    mut queue: &'a mut Queue<Response, U2>,
) -> Result<Initialized<'a, Tx, Rx>, Error>
    where
        Tx: Write<u8>,
        Rx: Read<u8>,
        EnablePin: OutputPin,
        ResetPin: OutputPin,
{
    let mut buffer: [u8; 1024] = [0; 1024];
    let mut pos = 0;

    const READY: [u8; 7] = *b"ready\r\n";

    let mut counter = 0;

    enable_pin.set_high().map_err(|_| Error::UnableToInitialize)?;
    reset_pin.set_high().map_err(|_| Error::UnableToInitialize)?;

    loop {
        let result = rx.read();
        match result {
            Ok(c) => {
                buffer[pos] = c;
                pos += 1;
                if pos >= READY.len() {
                    if buffer[pos - READY.len()..pos] == READY {
                        log::debug!("adapter is ready");
                        disable_echo(&mut tx, &mut rx);
                        enable_mux(&mut tx, &mut rx);
                        set_recv_mode(&mut tx, &mut rx);
                        //enable_mux();
                        //let mut queue = Queue::new();
                        let (producer, consumer) = queue.split();
                        //let (producer, consumer) = queue.split();
                        return Ok(
                            (
                                Adapter {
                                    tx,
                                    consumer,
                                },
                                Ingress::new(rx, producer),
                            )
                        );
                    }
                }
            }
            Err(e) => {
                if let nb::Error::WouldBlock = e {
                    continue;
                }
                counter += 1;
                if counter > 10_000 {
                    break;
                }
            }
        }
    }

    Err(Error::UnableToInitialize)
}

fn disable_echo<Tx, Rx>(tx: &mut Tx, rx: &mut Rx)
    where
        Tx: Write<u8>,
        Rx: Read<u8>,
{
    const cmd: &[u8] = b"ATE0\r\n";
    for b in cmd.iter() {
        nb::block!(tx.write(*b));
    }
    wait_for_ok(rx);
    info!("echo disabled");
}

fn enable_mux<Tx, Rx>(tx: &mut Tx, rx: &mut Rx)
    where
        Tx: Write<u8>,
        Rx: Read<u8>,
{
    const cmd: &[u8] = b"AT+CIPMUX=1\r\n";
    for b in cmd.iter() {
        nb::block!(tx.write(*b));
    }
    wait_for_ok(rx);
    info!("mux enabled");
}

fn set_recv_mode<Tx, Rx>(tx: &mut Tx, rx: &mut Rx)
    where
        Tx: Write<u8>,
        Rx: Read<u8>,
{
    const cmd: &[u8] = b"AT+CIPRECVMODE=1\r\n";
    for b in cmd.iter() {
        nb::block!(tx.write(*b));
    }
    wait_for_ok(rx);
    info!("mux enabled");
}

fn wait_for_ok<Rx>(rx: &mut Rx)
    where
        Rx: Read<u8>,
{
    let mut buf: [u8; 64] = [0; 64];
    let mut pos = 0;

    loop {
        match nb::block!(rx.read()) {
            Ok(b) => {
                buf[pos] = b;
                pos += 1;
                if buf[0..pos].ends_with(b"OK\r\n") {
                    log::info!( "matched OK");
                    return;
                }
            }
            Err(_) => {}
        }
    }
}

pub struct Adapter<'a, Tx>
    where
        Tx: Write<u8>,
{
    tx: Tx,
    consumer: Consumer<'a, Response, U2>,
}

impl<'a, Tx> Adapter<'a, Tx>
    where
        Tx: Write<u8>,
{
    pub fn send<'c>(&mut self, command: Command<'c>) -> Result<Response, Error> {
        let bytes = command.as_bytes();

        info!("writing command {}", core::str::from_utf8(bytes.as_bytes()).unwrap());
        // send the bytes of the command
        for b in bytes.as_bytes().iter() {
            nb::block!( self.tx.write(*b ) );
        }
        nb::block!( self.tx.write( b'\r' ));
        nb::block!( self.tx.write( b'\n' ));
        info!("await response");

        self.wait_for_response()
            /*
        loop {
            // busy loop until a response is received.
            let response = self.consumer.dequeue();
            match response {
                None => {
                    //info!("got a none");
                    continue;
                }
                Some(response) => {
                    info!("got {:?}", response);
                    return Ok(response);
                }
            }
        }
             */
        //command.parse()
        //Ok(Response::Ok)
    }

    fn wait_for_response(&mut self) -> Result<Response, Error> {
        loop {
            // busy loop until a response is received.
            let response = self.consumer.dequeue();
            match response {
                None => {
                    //info!("got a none");
                    continue;
                }
                Some(response) => {
                    //info!("got {:?}", response);
                    return Ok(response);
                }
            }
        }

    }

    pub fn get_firmware_info(&mut self) -> Result<FirmwareInfo, ()> {
        let command = Command::QueryFirmwareInfo;

        match self.send(command) {
            Ok(response) => {
                match response {
                    Response::FirmwareInfo(info) => {
                        Ok(info)
                    }
                    _ => {
                        Err(())
                    }
                }
            }
            Err(_) => {
                Err(())
            }
        }
    }

    pub fn get_ip_address(&mut self) -> Result<IpAddresses, ()> {
        let command = Command::QueryIpAddress;

        match self.send(command) {
            Ok(response) => {
                match response {
                    Response::IpAddresses(addresses) => {
                        Ok(addresses)
                    }
                    _ => {
                        Err(())
                    }
                }
            }
            Err(_) => {
                Err(())
            }
        }
    }

    pub fn join<'c>(&mut self, ssid: &'c str, password: &'c str) -> Result<(), WifiConnectionFailure> {
        let command = Command::JoinAp {
            ssid,
            password,
        };

        match self.send(command) {
            Ok(response) => {
                match response {
                    Response::Ok => {
                        Ok(())
                    }
                    Response::WifiConnectionFailure(reason) => {
                        Err(reason)
                    }
                    _ => {
                        Err(WifiConnectionFailure::ConnectionFailed)
                    }
                }
            }
            Err(_) => {
                Err(WifiConnectionFailure::ConnectionFailed)
            }
        }
    }

    pub fn connect_tcp(&mut self, link_id: usize, remote: SocketAddr) -> Result<(), ()> {
        let command = Command::StartConnection(link_id,
                                               ConnectionType::TCP,
                                               remote);
        match self.send(command) {
            Ok(response) => {
                match response {
                    Response::Ok => {
                        Ok(())
                    }
                    _ => {
                        Err(())
                    }
                }
            }
            Err(_) => {
                Err(())
            }
        }
    }

    pub fn write(&mut self, link_id: usize, buffer: &[u8]) -> Result<usize, ()> {
        let command = Command::Send {
            link_id,
            len: buffer.len(),
        };

        match self.send(command) {
            Ok(response) => {
                match response {
                    Response::Ok => {
                        //Ok(buffer.len())
                        match self.wait_for_response() {
                            Ok(response) => {
                                match response {
                                    Response::ReadyForData => {
                                        info!( "sending data {}", buffer.len());
                                        for b in buffer.iter() {
                                            nb::block!( self.tx.write( *b ));
                                        }
                                        info!( "sent data {}", buffer.len());
                                        match self.wait_for_response() {
                                            Ok(response) => {
                                                match response {
                                                    Response::SendOk(len) => {
                                                        Ok(len)
                                                    }
                                                    _ => {
                                                        Err(())
                                                    }
                                                }
                                            },
                                            Err(_) => {
                                                Err(())
                                            },
                                        }
                                    }
                                    _ => {
                                        Err(())
                                    }
                                }
                            },
                            Err(_) => {
                                Err(())
                            },
                        }
                    }
                    _ => {
                        Err(())
                    }
                }
            }
            Err(_) => {
                Err(())
            }
        }
    }

    pub fn read(&mut self, link_id: usize, buffer: &mut [u8]) -> Result<usize, ()> {
        let command = Command::Receive {
            link_id,
            len: buffer.len(),
        };

        match self.send(command) {
            Ok(response) => {
                match response {
                    Response::DataReceived(inbound, len) => {
                        for (i, b) in inbound[0..len].iter().enumerate() {
                            buffer[i] = *b;
                        }
                        Ok(len)
                    }
                    _ => {
                        Err(())
                    }
                }
            },
            Err(_) => {
                Err(())
            },
        }
    }



    pub fn into_network_stack<NumSockets, InboundBufSize>(self, sockets: &'a mut Sockets<NumSockets, InboundBufSize>) -> NetworkStack<'a, Tx, NumSockets, InboundBufSize>
        where
            NumSockets: ArrayLength<Socket<InboundBufSize>>,
            InboundBufSize: ArrayLength<u8>
    {
        NetworkStack::new(self, sockets)
    }
}

