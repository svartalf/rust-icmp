
use std::net::IpAddr;
use std::io::{Result};
use std::time::Duration;

use libc as c;

use compat::{AsInner, set_timeout, timeout};

use sys::Socket;

/// An Internet Control Message Protocol socket.
///
/// This is an implementation of a bound ICMP socket. This supports both IPv4 and
/// IPv6 addresses, and there is no corresponding notion of a server because ICMP
/// is a datagram protocol.
///
/// TODO: Example
pub struct IcmpSocket {
    inner: Socket,
}

impl IcmpSocket {

    pub fn connect(addr: IpAddr) -> Result<IcmpSocket> {
        let inner = Socket::connect(addr)?;

        Ok(IcmpSocket {
            inner: inner,
        })
    }

    /// Receives data from the socket. On success, returns the number of bytes read.
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        self.inner.recv(buf)
    }

    /// Receives data from the socket. On success, returns the number of bytes
    /// read and the address from whence the data came.
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, IpAddr)> {
        self.inner.recv_from(buf)
    }

    /// Sends data on the socket to the remote address to which it is connected.
    ///
    /// The `connect` method will connect this socket to a remote address. This
    /// method will fail if the socket is not connected.
    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.send(buf)
    }

    /// Sets the read timeout to the timeout specified.
    ///
    /// If the value specified is `None`, then `read` calls will block
    /// indefinitely. It is an error to pass the zero `Duration` to this
    /// method.
    ///
    /// # Note
    ///
    /// Platforms may return a different error code whenever a read times out as
    /// a result of setting this option. For example Unix typically returns an
    /// error of the kind `WouldBlock`, but Windows may return `TimedOut`.
    pub fn set_read_timeout(&self, dur: Option<Duration>) -> Result<()> {
        set_timeout(self.as_inner(), dur, c::SO_RCVTIMEO)
    }

    /// Sets the write timeout to the timeout specified.
    ///
    /// If the value specified is `None`, then `write` calls will block
    /// indefinitely. It is an error to pass the zero `Duration` to this
    /// method.
    ///
    /// # Note
    ///
    /// Platforms may return a different error code whenever a write times out
    /// as a result of setting this option. For example Unix typically returns
    /// an error of the kind `WouldBlock`, but Windows may return `TimedOut`.
    pub fn set_write_timeout(&self, dur: Option<Duration>) -> Result<()> {
        set_timeout(self.as_inner(), dur, c::SO_SNDTIMEO)
    }

    /// Returns the read timeout of this socket.
    ///
    /// If the timeout is `None`, then `read` calls will block indefinitely.
    pub fn read_timeout(&self) -> Result<Option<Duration>> {
        timeout(self.as_inner(), c::SO_RCVTIMEO)
    }

    /// Returns the write timeout of this socket.
    ///
    /// If the timeout is `None`, then `write` calls will block indefinitely.
    pub fn write_timeout(&self) -> Result<Option<Duration>> {
        timeout(self.as_inner(), c::SO_SNDTIMEO)
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.inner.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`set_ttl`][link].
    ///
    /// [link]: #method.set_ttl
    pub fn ttl(&self) -> Result<u32> {
        self.inner.ttl()
    }

    /// Sets the value of the SO_BROADCAST option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast address.
    pub fn set_broadcast(&self, broadcast: bool) -> Result<()> {
        self.inner.set_broadcast(broadcast)
    }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see
    /// [`set_broadcast`][link].
    ///
    /// [link]: #method.set_broadcast
    pub fn broadcast(&self) -> Result<bool> {
        self.inner.broadcast()
    }

    /// Sets the QoS value of the `IP_TOS`/`IPV6_TCLASS` option for this socket.
    ///
    /// This value sets the TOS/DSCP field that is used in every packet sent
    /// from this socket.
    pub fn set_qos(&self, qos: u8) -> Result<()> {
        self.inner.set_qos(qos)
    }

    /// Gets the value of the `IP_TOS`/`IPV6_TCLASS` option for this socket.
    ///
    /// For more information about this option, see
    /// [`set_qos`][link].
    ///
    /// [link]: #method.set_qos
    pub fn qos(&self) -> Result<u8> {
        self.inner.qos()
    }

}

impl AsInner<Socket> for IcmpSocket {
    fn as_inner(&self) -> &Socket {
        &self.inner
    }
}