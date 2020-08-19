use crate::adapter::Adapter;
use embedded_hal::{
    digital::v2::OutputPin,
    serial::Write,
};

use heapless::ArrayLength;
use heapless::Vec;
use embedded_nal::{TcpStack, SocketAddr, Mode};
use core::cell::RefCell;
use crate::network::Error::{SocketNotOpen, UnableToOpen, WriteError, ReadError};
use crate::protocol::{Command, ConnectionType};

#[derive(Debug)]
pub enum SocketState {
    Closed,
    Open,
    Connected,
}

pub struct Socket<N>
    where
        N: ArrayLength<u8>
{
    state: SocketState,
    inbound_buffer: Vec<u8, N>,
}

impl<N> Socket<N>
    where
        N: ArrayLength<u8>
{
    pub fn new() -> Self {
        Self {
            state: SocketState::Closed,
            inbound_buffer: Vec::new(),
        }
    }
}

pub struct Sockets<NumSockets, BufferSize>
    where
        NumSockets: ArrayLength<Socket<BufferSize>>,
        BufferSize: ArrayLength<u8>,
{
    pub sockets: Vec<Socket<BufferSize>, NumSockets>,
}

pub struct NetworkStack<'a, Tx, NumSockets, InboundBufSize>
    where
        Tx: Write<u8>,
        NumSockets: ArrayLength<Socket<InboundBufSize>>,
        InboundBufSize: ArrayLength<u8>,
{
    adapter: RefCell<Adapter<'a, Tx>>,
    sockets: RefCell<&'a mut Sockets<NumSockets, InboundBufSize>>,
}

impl<'a, Tx, NumSockets, InboundBufSize> NetworkStack<'a, Tx, NumSockets, InboundBufSize>
    where
        Tx: Write<u8>,
        NumSockets: ArrayLength<Socket<InboundBufSize>>,
        InboundBufSize: ArrayLength<u8>,
{
    pub(crate) fn new(adapter: Adapter<'a, Tx>, sockets: &'a mut Sockets<NumSockets, InboundBufSize>) -> Self {
        Self {
            adapter: RefCell::new(adapter),
            sockets: RefCell::new(sockets),
        }
    }
}

#[derive(Debug)]
pub struct TcpSocket(usize);

#[derive(Debug)]
pub enum Error {
    NoAvailableSockets,
    SocketNotOpen,
    UnableToOpen,
    WriteError,
    ReadError,
}

impl<'a, Tx, NumSockets, InboundBufSize> TcpStack for NetworkStack<'a, Tx, NumSockets, InboundBufSize>
    where
        Tx: Write<u8>,
        NumSockets: ArrayLength<Socket<InboundBufSize>>,
        InboundBufSize: ArrayLength<u8>,
{
    type TcpSocket = TcpSocket;
    type Error = Error;

    fn open(&self, mode: Mode) -> Result<Self::TcpSocket, Self::Error> {
        let mut sockets = self.sockets.borrow_mut();
        let socket = sockets.sockets.iter_mut().enumerate().find(|e| {
            if let SocketState::Closed = e.1.state {
                true
            } else {
                false
            }
        });

        match socket {
            None => {
                Err(Error::NoAvailableSockets)
            }
            Some(socket) => {
                socket.1.state = SocketState::Open;
                Ok(TcpSocket(socket.0))
            }
        }
    }

    fn connect(&self, socket: Self::TcpSocket, remote: SocketAddr) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        match adapter.connect_tcp(socket.0, remote) {
            Ok(_) => {
                Ok(socket)
            }
            Err(_) => {
                Err(UnableToOpen)
            }
        }
    }

    fn is_connected(&self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn write(&self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        match adapter.write(socket.0, buffer) {
            Ok(_) => {
                Ok(buffer.len())
            }
            Err(_) => {
                nb::Result::Err(nb::Error::Other(WriteError))
            }
        }
    }

    fn read(&self, socket: &mut Self::TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        log::info!("try read");
        let result = adapter.read(socket.0, buffer);
        let result = match result {
            Ok(len) => {
                Ok(len)
            }
            Err(_) => {
                Err(nb::Error::Other(ReadError))
            }
        };
        log::info!("read done");
        result
    }

    fn close(&self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        let mut sockets = self.sockets.borrow_mut();
        let mut socket = sockets.sockets.get_mut(socket.0).unwrap();
        match socket.state {
            SocketState::Closed => {
                Err(Error::SocketNotOpen)
            }
            SocketState::Open => {
                socket.state = SocketState::Closed;
                Ok(())
            }
            SocketState::Connected => {
                socket.state = SocketState::Closed;
                Ok(())
            }
        }
    }
}
