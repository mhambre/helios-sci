//! Network-related functionality.
//! This module contains types and functions for working with network connections, such as TCP and ~~UDP~~ sockets.

use std::io::{Read, Write};
use std::net::{SocketAddr as StdSocketAddr, TcpListener as StdTcpListener, TcpStream as StdTcpStream};

use crate::error;
use crate::net::SocketAddr;

/// A TCP connection.
pub struct TcpConnection {
    /// Remote peer address of this TCP connection.
    addr: SocketAddr,
    connection: StdTcpStream,
}

/// A TCP listener.
pub struct TcpListener {
    /// Local address that this listener is bound to.
    addr: SocketAddr,
    listener: StdTcpListener,
}

impl TcpConnection {
    /// Binds a TCP connection to the specified address.
    pub fn connect(addr: SocketAddr) -> Result<TcpConnection, error::net::TcpError> {
        let std_addr = StdSocketAddr::try_from(addr.clone()).map_err(|_| error::net::TcpError::BindFailed)?;
        let conn = StdTcpStream::connect(std_addr).map_err(|_| error::net::TcpError::ConnectionFailed)?;
        Ok(TcpConnection { addr, connection: conn })
    }

    /// Writes data to the TCP connection.
    pub fn write(&mut self, data: &[u8]) -> Result<usize, error::net::TcpError> {
        self.connection.write(data).map_err(|_| error::net::TcpError::WriteFailed)
    }

    /// Reads data from the TCP connection.
    pub fn read(&mut self, data: &mut [u8]) -> Result<usize, error::net::TcpError> {
        self.connection.read(data).map_err(|_| error::net::TcpError::ReadFailed)
    }

    /// Returns the address of this remote TCP connection.
    pub fn peer_addr(&self) -> &SocketAddr {
        &self.addr
    }
}

impl TcpListener {
    /// Binds a TCP listener to the specified address.
    pub fn bind(addr: SocketAddr) -> Result<TcpListener, error::net::TcpError> {
        let std_addr = StdSocketAddr::try_from(addr.clone()).map_err(|_| error::net::TcpError::BindFailed)?;
        let listener = StdTcpListener::bind(std_addr).map_err(|_| error::net::TcpError::BindFailed)?;
        let local = listener.local_addr().map_err(|_| error::net::TcpError::BindFailed)?;
        let local_addr = SocketAddr::new(local.ip().to_string(), local.port());
        Ok(TcpListener {
            addr: local_addr,
            listener,
        })
    }

    /// Accepts a new incoming TCP connection.
    pub fn accept(&self) -> Result<TcpConnection, error::net::TcpError> {
        let (conn, addr) = self.listener.accept().map_err(|_| error::net::TcpError::ConnectionFailed)?;
        let socket_addr = SocketAddr::new(addr.ip().to_string(), addr.port());
        Ok(TcpConnection {
            addr: socket_addr,
            connection: conn,
        })
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

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn listener_bind_with_ephemeral_port_reports_assigned_port() {
        let listener = TcpListener::bind(SocketAddr::new("127.0.0.1".to_string(), 0)).expect("listener should bind");

        assert_eq!(listener.local_addr().ip(), "127.0.0.1");
        assert_ne!(listener.local_addr().port(), 0);
    }

    #[test]
    fn connect_accept_and_io_round_trip() {
        let listener = TcpListener::bind(SocketAddr::new("127.0.0.1".to_string(), 0)).expect("listener should bind");
        let listener_addr = listener.local_addr().clone();

        let server = thread::spawn(move || {
            let mut conn = listener.accept().expect("server should accept");
            let mut buf = [0_u8; 4];
            let n = conn.read(&mut buf).expect("server should read");
            assert_eq!(&buf[..n], b"ping");
            let written = conn.write(b"pong").expect("server should write");
            assert_eq!(written, 4);
        });

        let mut client = TcpConnection::connect(listener_addr).expect("client should connect");
        let written = client.write(b"ping").expect("client should write");
        assert_eq!(written, 4);

        let mut buf = [0_u8; 4];
        let n = client.read(&mut buf).expect("client should read");
        assert_eq!(&buf[..n], b"pong");

        server.join().expect("server thread should finish");
    }
}
