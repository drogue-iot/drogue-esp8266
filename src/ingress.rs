use heapless::Vec;
use crate::{
    buffer::Buffer,
    protocol::Response,
};
use heapless::{
    consts::U2,
    spsc::Producer,
};
use embedded_hal::serial::Read;

use log::info;


pub struct Ingress<'a, Rx>
    where
        Rx: Read<u8>,
{
    rx: Rx,
    producer: Producer<'a, Response, U2>,
    buffer: Buffer,
}

impl<'a, Rx> Ingress<'a, Rx>
    where
        Rx: Read<u8>,
{
    pub fn new(rx: Rx, producer: Producer<'a, Response, U2>) -> Self {
        Self {
            rx,
            producer,
            buffer: Buffer::new(),
        }
    }

    pub fn isr(&mut self) {
        if let Ok(d) = self.rx.read() {
            self.write(d);
            //info!( "{}", d as char);
        }
    }

    pub fn write(&mut self, octet: u8) -> Result<(), u8> {
        self.buffer.write(octet)?;
        Ok(())
    }

    pub fn digest(&mut self) {
        let result = self.buffer.parse();
        //info!( "digest {:?}", result);

        match result {
            Ok(response) => {
                match response {
                    Response::None => {}

                    //response @ _ => {
                    //info!("response {:?}", *response);
                    //   self.producer.enqueue(response);
                    Response::Ok => {
                        self.producer.enqueue(response);
                    }
                    Response::FirmwareInfo(..) => {
                        self.producer.enqueue(response);
                    }
                    Response::ReadyForData => {}
                    Response::WifiConnected => {
                        log::info!("wifi connected");
                    }
                    Response::WifiConnectionFailure(..) => {
                        self.producer.enqueue(response);
                    }
                    Response::WifiDisconnect => {
                        log::info!("wifi disconnect");
                    }
                    Response::GotIp => {
                        log::info!("wifi got ip");
                    }
                    Response::IpAddresses(..) => {
                        self.producer.enqueue(response);
                    }
                    Response::Connect(..) => {
                        log::info!("connect {:?}", response);
                        self.producer.enqueue(Response::Ok);
                    }
                }
            }

            Err(e) => {}
        }
    }
}