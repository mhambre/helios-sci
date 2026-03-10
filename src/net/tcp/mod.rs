use cfg_if::cfg_if;

use crate::error;
use crate::net::SocketAddr;

cfg_if! {
    if #[cfg(all(target_arch = "x86_64", target_os = "helios"))] {
        mod helios;
        use helios as backend;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        use linux as backend;
    } else {
        compile_error!("Unsupported Target OS");
    }
}

/// A TCP connection.
pub struct TcpConnection {
    inner: backend::TcpConnection,
}

/// A TCP listener.
pub struct TcpListener {
    inner: backend::TcpListener,
}

impl TcpConnection {
    /// Connects a TCP connection to the specified address.
    pub fn connect(addr: SocketAddr) -> Result<TcpConnection, error::net::TcpError> {
        let inner = backend::TcpConnection::connect(addr)?;
        Ok(TcpConnection { inner })
    }

    /// Writes data to the TCP connection.
    pub fn write(&mut self, data: &[u8]) -> Result<usize, error::net::TcpError> {
        self.inner.write(data)
    }

    /// Reads data from the TCP connection.
    pub fn read(&mut self, data: &mut [u8]) -> Result<usize, error::net::TcpError> {
        self.inner.read(data)
    }

    /// Returns the address of this remote TCP connection.
    pub fn peer_addr(&self) -> &SocketAddr {
        self.inner.peer_addr()
    }
}

impl TcpListener {
    /// Binds a TCP listener to the specified address.
    pub fn bind(addr: SocketAddr) -> Result<TcpListener, error::net::TcpError> {
        let inner = backend::TcpListener::bind(addr)?;
        Ok(TcpListener { inner })
    }

    /// Accepts a new incoming TCP connection.
    pub fn accept(&self) -> Result<TcpConnection, error::net::TcpError> {
        let inner = self.inner.accept()?;
        Ok(TcpConnection { inner })
    }

    /// Returns an iterator over incoming TCP connections.
    pub fn incoming(&self) -> impl Iterator<Item = Result<TcpConnection, error::net::TcpError>> + '_ {
        self.inner.incoming().map(|res| res.map(|inner| TcpConnection { inner }))
    }

    /// Returns the local address that this listener is bound to.
    pub fn local_addr(&self) -> &SocketAddr {
        self.inner.local_addr()
    }
}
