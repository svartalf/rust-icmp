
use std::net::IpAddr;
use std::io::{Result, ErrorKind};
use std::mem;

use libc as c;

use compat::{IntoInner, FromInner, AsInner, cvt, setsockopt, getsockopt};

const IPPROTO_ICMP: c::c_int = 1;

#[cfg(target_os = "linux")]
use libc::SOCK_CLOEXEC;
#[cfg(not(target_os = "linux"))]
const SOCK_CLOEXEC: c::c_int = 0;


/// An Internet Control Message Protocol socket.
///
/// This is an implementation of a bound ICMP socket. This supports both IPv4 and
/// IPv6 addresses, and there is no corresponding notion of a server because ICMP
/// is a datagram protocol.
///
/// TODO: Example
pub struct IcmpSocket {
    fd: c::c_int,
    peer: c::sockaddr,
}

impl IcmpSocket {

    pub fn connect(addr: IpAddr) -> Result<IcmpSocket> {
        let family = match addr {
            IpAddr::V4(..) => c::AF_INET,
            IpAddr::V6(..) => c::AF_INET6,
        };

        let fd = unsafe {
            cvt(c::socket(family, c::SOCK_RAW | SOCK_CLOEXEC, IPPROTO_ICMP))?
        };

        Ok(IcmpSocket {
            fd: fd,
            peer: addr.into_inner(),
        })
    }

    /// Receives data from the socket. On success, returns the number of bytes read.
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let ret = unsafe {
            cvt(c::recv(
                    self.fd,
                    buf.as_mut_ptr() as *mut c::c_void,
                    buf.len() as c::size_t,
                    0,
            ))
        };

        match ret {
            Ok(size) => Ok(size as usize),
            Err(ref err) if err.kind() == ErrorKind::Interrupted => Ok(0),
            Err(err) => Err(err),
        }
    }

    /// Receives data from the socket. On success, returns the number of bytes
    /// read and the address from whence the data came.
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, IpAddr)> {
        let mut peer: c::sockaddr = unsafe { mem::uninitialized() };
        let ret = unsafe {
            cvt(c::recvfrom(
                    self.fd,
                    buf.as_mut_ptr() as *mut c::c_void,
                    buf.len() as c::size_t,
                    0,
                    &mut peer,
                    &mut (mem::size_of_val(&peer) as c::socklen_t)
                )
            )
        };

        match ret {
            Ok(size) => Ok((size as usize, IpAddr::from_inner(peer))),
            Err(ref err) if err.kind() == ErrorKind::Interrupted => Ok((0, IpAddr::from_inner(peer))),
            Err(err) => Err(err),
        }
    }

    /// Sends data on the socket to the remote address to which it is connected.
    ///
    /// The `connect` method will connect this socket to a remote address. This
    /// method will fail if the socket is not connected.
    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe {
            cvt(c::sendto(
                    self.fd,
                    buf.as_ptr() as *mut c::c_void,
                    buf.len() as c::size_t,
                    0,
                    &self.peer,
                    mem::size_of_val(&self.peer) as c::socklen_t,
                )
            )?
        };

        Ok(ret as usize)
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        setsockopt(self, c::IPPROTO_IP, c::IP_TTL, ttl as c::c_int)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`set_ttl`][link].
    ///
    /// [link]: #method.set_ttl
    pub fn ttl(&self) -> Result<u32> {
        getsockopt(self, c::IPPROTO_IP, c::IP_TTL)
    }

    /// Sets the value of the SO_BROADCAST option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast address.
    pub fn set_broadcast(&self, broadcast: bool) -> Result<()> {
        setsockopt(&self, c::SOL_SOCKET, c::SO_BROADCAST, broadcast as c::c_int)
    }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see
    /// [`set_broadcast`][link].
    ///
    /// [link]: #method.set_broadcast
    pub fn broadcast(&self) -> Result<bool> {
        let raw: c::c_int = getsockopt(&self, c::SOL_SOCKET, c::SO_BROADCAST)?;
        Ok(raw != 0)
    }

}

impl Drop for IcmpSocket {
    fn drop(&mut self) {
        let _ = unsafe {
            c::close(self.fd)
        };
    }
}

impl AsInner<c::c_int> for IcmpSocket {
    fn as_inner(&self) -> &c::c_int {
        &self.fd
    }
}