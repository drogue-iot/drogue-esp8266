use crate::adapter::{Adapter, SocketError};
use embedded_hal::{
    digital::v2::OutputPin,
    serial::Write,
};

use heapless::ArrayLength;
use heapless::Vec;
use drogue_network::{TcpStack, SocketAddr, Mode};
use core::cell::RefCell;
use crate::protocol::{Command, ConnectionType};
use crate::adapter::SocketError::{NoAvailableSockets, UnableToOpen, WriteError, ReadError};

/// NetworkStack for and ESP8266
pub struct NetworkStack<'a, Tx>
    where
        Tx: Write<u8>,
{
    adapter: RefCell<Adapter<'a, Tx>>,
}

impl<'a, Tx> NetworkStack<'a, Tx>
    where
        Tx: Write<u8>,
{
    pub(crate) fn new(adapter: Adapter<'a, Tx>) -> Self {
        Self {
            adapter: RefCell::new(adapter),
        }
    }
}

/// Handle to a socket.
#[derive(Debug)]
pub struct TcpSocket{
    link_id: usize,
    mode: Mode,
}

impl<'a, Tx> TcpStack for NetworkStack<'a, Tx>
    where
        Tx: Write<u8>,
{
    type TcpSocket = TcpSocket;
    type Error = super::adapter::SocketError;

    fn open(&self, mode: Mode) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        Ok( TcpSocket{ link_id: adapter.open()?, mode })
    }

    fn connect(&self, socket: Self::TcpSocket, remote: SocketAddr) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        adapter.connect_tcp(socket.link_id, remote)?;
        Ok(socket)
    }

    fn is_connected(&self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn write(&self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        Ok(adapter.write(socket.link_id, buffer).map_err(|e| nb::Error::from(e))?)
    }

    fn read(&self, socket: &mut Self::TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        match socket.mode {
            Mode::Blocking => {
                nb::block!(adapter.read(socket.link_id, buffer)).map_err(|e| nb::Error::from(e))
            },
            Mode::NonBlocking => {
                adapter.read(socket.link_id, buffer)
            },
            Mode::Timeout(_) => {
                unimplemented!()
            },
        }
    }

    fn close(&self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        adapter.close(socket.link_id)
    }
}
