//! Errors related to network operations.

/// Errors related to TCP connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpError {
    /// Binding to a local address failed.
    BindFailed,
    /// The connection failed to establish.
    ConnectionFailed,
    /// Writing to the connection failed.
    WriteFailed,
    /// Reading from the connection failed.
    ReadFailed,
    /// The connection was lost.
    Timeout,
}

/// Errors related to network addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
    /// The address is invalid.
    InvalidAddress,
}
