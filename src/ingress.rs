use crate::{buffer::Buffer, protocol::Response};
use heapless::{
    consts::{U16, U2},
    spsc::Producer,
};

use embedded_hal::serial::Read;
use nb::Error;
use core::fmt::Debug;

pub struct Ingress<'a, Rx>
    where
        Rx: Read<u8>,
{
    rx: Rx,
    response_producer: Producer<'a, Response, U2>,
    notification_producer: Producer<'a, Response, U16>,
    buffer: Buffer,
}

impl<'a, Rx> Ingress<'a, Rx>
    where
        Rx: Read<u8>,
{
    pub fn new(
        rx: Rx,
        response_producer: Producer<'a, Response, U2>,
        notification_producer: Producer<'a, Response, U16>,
    ) -> Self {
        Self {
            rx,
            response_producer,
            notification_producer,
            buffer: Buffer::new(),
        }
    }

    /// Method to be called from USART or appropriate ISR.
    pub fn isr(&mut self) -> Result<(), u8> {
        let mut count = 0;
        loop {
            let result = self.rx.read();
            match result {
                Ok(d) => {
                    self.write(d)?;
                    count +=1;
                }
                Err(e) => {
                    match e {
                        Error::Other(o) => {
                        }
                        Error::WouldBlock => {
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn write(&mut self, octet: u8) -> Result<(), u8> {
        self.buffer.write(octet)?;
        Ok(())
    }

    /// Digest and process the existing ingressed buffer to
    /// emit appropriate responses and notifications back
    pub fn digest(&mut self) {
        let result = self.buffer.parse();

        if let Ok(response) = result {
            match response {
                Response::None => {}
                Response::Ok
                | Response::Error
                | Response::FirmwareInfo(..)
                | Response::Connect(..)
                | Response::ReadyForData
                | Response::DataReceived(..)
                | Response::SendOk(..)
                | Response::WifiConnectionFailure(..)
                | Response::IpAddresses(..) => {
                    if let Err(response) = self.response_producer.enqueue(response) {
                        log::error!("failed to enqueue response {:?}", response);
                    }
                }
                Response::Closed(..) | Response::DataAvailable { .. } => {
                    if let Err(response) = self.notification_producer.enqueue(response) {
                        log::error!("failed to enqueue notification {:?}", response);
                    }
                }
                Response::WifiConnected => {
                    log::info!("wifi connected");
                }
                Response::WifiDisconnect => {
                    log::info!("wifi disconnect");
                }
                Response::GotIp => {
                    log::info!("wifi got ip");
                }
            }
        }
    }
}
