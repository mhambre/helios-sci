//! Network-related functionality.
//! This module contains types and functions for working with network connections, such as TCP and ~~UDP~~ sockets.

#[cfg(not(all(target_arch = "x86", target_os = "helios")))]
use std::net::{SocketAddr as StdSocketAddr, TcpListener as StdTcpListener, TcpStream as StdTcpStream};

use crate::error;
use crate::net::SocketAddr;

/// A TCP connection.
pub struct TcpConnection {
    /// Remote peer address of this TCP connection.
    addr: SocketAddr,
}

/// A TCP listener.
pub struct TcpListener {
    /// Local address that this listener is bound to.
    addr: SocketAddr,
}

impl TcpConnection {
    /// Connects a TCP connection to the specified address.
    pub fn bind(_addr: SocketAddr) -> Result<TcpConnection, error::net::TcpError> {
        unimplemented!("TCP connections are not supported yet on i686-helios")
    }

    /// Writes data to the TCP connection.
    pub fn write(&mut self, data: &[u8]) -> Result<usize, error::net::TcpError> {
        unimplemented!("TCP connections are not supported yet on i686-helios")
    }

    /// Reads data from the TCP connection.
    pub fn read(&mut self, data: &mut [u8]) -> Result<usize, error::net::TcpError> {
        unimplemented!("TCP connections are not supported yet on i686-helios")
    }

    /// Returns the address of this remote TCP connection.
    pub fn peer_addr(&self) -> &SocketAddr {
        &self.addr
    }
}

impl TcpListener {
    /// Binds a TCP listener to the specified address.
    pub fn bind(_addr: SocketAddr) -> Result<TcpListener, error::net::TcpError> {
        unimplemented!("TCP listeners are not supported yet on i686-helios")
    }

    /// Accepts a new incoming TCP connection.
    pub fn accept(&self) -> Result<TcpConnection, error::net::TcpError> {
        unimplemented!("TCP listeners are not supported yet on i686-helios")
    }

    /// Returns an iterator over incoming TCP connections.
    pub fn incoming(&self) -> impl Iterator<Item = Result<TcpConnection, error::net::TcpError>> + '_ {
        std::iter::repeat_with(|| self.accept())
    }

    /// Returns the local address that this listener is bound to.
    pub fn local_addr(&self) -> &SocketAddr {
        &self.addr
    }
}
