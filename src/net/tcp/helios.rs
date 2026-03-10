//! Network-related functionality.
//! This module contains types and functions for working with network connections, such as TCP and ~~UDP~~ sockets.

use crate::error;
use crate::net::SocketAddr;

/// Helios backend TCP connection.
pub(crate) struct TcpConnection {
    /// Remote peer address of this TCP connection.
    addr: SocketAddr,
}

/// Helios backend TCP listener.
pub(crate) struct TcpListener {
    /// Local address that this listener is bound to.
    addr: SocketAddr,
}

impl TcpConnection {
    /// Connects a TCP connection to the specified address.
    pub(crate) fn connect(_addr: SocketAddr) -> Result<TcpConnection, error::net::TcpError> {
        unimplemented!("TCP connections are not supported yet on x86_64-helios")
    }

    /// Writes data to the TCP connection.
    pub(crate) fn write(&mut self, _data: &[u8]) -> Result<usize, error::net::TcpError> {
        unimplemented!("TCP connections are not supported yet on x86_64-helios")
    }

    /// Reads data from the TCP connection.
    pub(crate) fn read(&mut self, _data: &mut [u8]) -> Result<usize, error::net::TcpError> {
        unimplemented!("TCP connections are not supported yet on x86_64-helios")
    }

    /// Returns the address of this remote TCP connection.
    pub(crate) fn peer_addr(&self) -> &SocketAddr {
        &self.addr
    }
}

impl TcpListener {
    /// Binds a TCP listener to the specified address.
    pub(crate) fn bind(_addr: SocketAddr) -> Result<TcpListener, error::net::TcpError> {
        unimplemented!("TCP listeners are not supported yet on x86_64-helios")
    }

    /// Accepts a new incoming TCP connection.
    pub(crate) fn accept(&self) -> Result<TcpConnection, error::net::TcpError> {
        unimplemented!("TCP listeners are not supported yet on x86_64-helios")
    }

    /// Returns an iterator over incoming TCP connections.
    pub(crate) fn incoming(&self) -> impl Iterator<Item = Result<TcpConnection, error::net::TcpError>> + '_ {
        std::iter::repeat_with(|| self.accept())
    }

    /// Returns the local address that this listener is bound to.
    pub(crate) fn local_addr(&self) -> &SocketAddr {
        &self.addr
    }
}
