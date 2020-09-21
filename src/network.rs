use crate::adapter::{Adapter, SocketError};
use embedded_hal::serial::Write;

use core::cell::RefCell;
use drogue_network::{Mode, SocketAddr, TcpStack, Dns, AddrType};
use core::fmt::Debug;
use nom::lib::std::fmt::Formatter;
use no_std_net::IpAddr;
use heapless::String;

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
pub struct TcpSocket {
    link_id: usize,
    mode: Mode,
}

impl Debug for TcpSocket {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TcpSocket")
            .field("link_id", &self.link_id)
            .field("mode",
                   & match self.mode {
                       Mode::Blocking => {
                           "blocking"
                       }
                       Mode::NonBlocking => {
                           "non-blocking"
                       }
                       Mode::Timeout(t) => {
                           "timeout"
                       }
                   },
            )
            .finish()
    }
}

impl<'a, Tx> TcpStack for NetworkStack<'a, Tx>
    where
        Tx: Write<u8>,
{
    type TcpSocket = TcpSocket;
    type Error = SocketError;

    fn open(&self, mode: Mode) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        Ok(TcpSocket {
            link_id: adapter.open()?,
            mode,
        })
    }

    fn connect(
        &self,
        socket: Self::TcpSocket,
        remote: SocketAddr,
    ) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        adapter.connect_tcp(socket.link_id, remote)?;
        Ok(socket)
    }

    fn is_connected(&self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        let adapter = self.adapter.borrow();
        adapter.is_connected(socket.link_id)
    }

    fn write(&self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        Ok(adapter
            .write(socket.link_id, buffer)
            .map_err(nb::Error::from)?)
    }

    fn read(
        &self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        match socket.mode {
            Mode::Blocking => {
                nb::block!(adapter.read(socket.link_id, buffer)).map_err(nb::Error::from)
            }
            Mode::NonBlocking => adapter.read(socket.link_id, buffer),
            Mode::Timeout(_) => unimplemented!(),
        }
    }

    fn close(&self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        adapter.close(socket.link_id)
    }
}

pub enum DnsError {
    NoSuchHost,
}

impl<'a, Tx> Dns for NetworkStack<'a, Tx>
    where
        Tx: Write<u8>,
{
    type Error = DnsError;

    fn gethostbyname(&self, hostname: &str, addr_type: AddrType) -> Result<IpAddr, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        adapter.get_host_by_name(hostname)
    }

    fn gethostbyaddr(&self, addr: IpAddr) -> Result<String<_>, Self::Error> {
        unimplemented!()
    }
}
