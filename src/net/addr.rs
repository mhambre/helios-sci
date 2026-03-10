//! Network address types and utilities.

use core::convert::TryFrom;
#[cfg(not(all(target_arch = "x86_64", target_os = "helios")))]
use std::net::SocketAddr as StdSocketAddr;

use crate::error;

/// Network socket address, consisting of an IP address and a port number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketAddr {
    ip: String,
    port: u16,
}

impl SocketAddr {
    /// Creates a new `SocketAddr` with the given IP address and port number.
    pub fn new(ip: String, port: u16) -> Self {
        Self { ip, port }
    }

    /// Returns the IP address of the socket address.
    pub fn ip(&self) -> &str {
        &self.ip
    }

    /// Returns the port number of the socket address.
    pub fn port(&self) -> u16 {
        self.port
    }
}

/// Converts a `&str` into a `SocketAddr`.
impl TryFrom<&str> for SocketAddr {
    type Error = error::net::AddressError;

    fn try_from(addr: &str) -> Result<Self, Self::Error> {
        let colon = addr.rfind(':').ok_or(error::net::AddressError::InvalidAddress)?;
        let (ip_part, port_part) = addr.split_at(colon);
        let port_str = &port_part[1..]; // skip ':'
        if ip_part.is_empty() || port_str.is_empty() {
            return Err(error::net::AddressError::InvalidAddress);
        }

        let port = port_str.parse().map_err(|_| error::net::AddressError::InvalidAddress)?;

        Ok(Self {
            ip: ip_part.to_string(), // requires alloc
            port,
        })
    }
}

/// Converts a `crate::SocketAddr` into a `std::SocketAddr`.
#[cfg(not(all(target_arch = "x86_64", target_os = "helios")))]
impl TryFrom<SocketAddr> for StdSocketAddr {
    type Error = error::net::AddressError;

    fn try_from(addr: SocketAddr) -> Result<Self, Self::Error> {
        format!("{}:{}", addr.ip(), addr.port())
            .parse::<StdSocketAddr>()
            .map_err(|_| error::net::AddressError::InvalidAddress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_address() {
        let addr = SocketAddr::try_from("127.0.0.1:8080").expect("address should parse");
        assert_eq!(addr.ip(), "127.0.0.1");
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn rejects_missing_port() {
        let err = SocketAddr::try_from("127.0.0.1").expect_err("address should be invalid");
        assert_eq!(err, error::net::AddressError::InvalidAddress);
    }

    #[test]
    fn rejects_empty_ip_or_port() {
        let left = SocketAddr::try_from(":8080").expect_err("empty IP should be invalid");
        let right = SocketAddr::try_from("127.0.0.1:").expect_err("empty port should be invalid");

        assert_eq!(left, error::net::AddressError::InvalidAddress);
        assert_eq!(right, error::net::AddressError::InvalidAddress);
    }

    #[test]
    fn rejects_non_numeric_port() {
        let err = SocketAddr::try_from("127.0.0.1:http").expect_err("port should be numeric");
        assert_eq!(err, error::net::AddressError::InvalidAddress);
    }

    #[cfg(not(all(target_arch = "x86_64", target_os = "helios")))]
    #[test]
    fn converts_to_std_socket_addr() {
        let addr = SocketAddr::new("127.0.0.1".to_string(), 3030);
        let std_addr = StdSocketAddr::try_from(addr).expect("conversion should succeed");
        assert_eq!(std_addr.to_string(), "127.0.0.1:3030");
    }
}
