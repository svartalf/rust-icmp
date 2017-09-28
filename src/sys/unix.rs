use std::net::{IpAddr, SocketAddr};
use std::io::{Result, ErrorKind, Error};
use std::os::unix::io::{RawFd, AsRawFd, IntoRawFd, FromRawFd};
use std::mem;

use libc as c;

use compat::{IntoInner, FromInner, AsInner, cvt, setsockopt, getsockopt};

// Following constants are not defined in libc (as for 0.2.31 version)
// Ipv4
#[cfg(target_os = "linux")]
const IP_TOS: c::c_int = 1;
#[cfg(target_os = "macos")]
const IP_TOS: c::c_int = 3;

// Ipv6
#[cfg(target_os = "linux")]
const IPV6_UNICAST_HOPS: c::c_int = 16;
#[cfg(target_os = "macos")]
const IPV6_UNICAST_HOPS: c::c_int = 4;

#[cfg(target_os = "linux")]
const IPV6_TCLASS: c::c_int = 67;
#[cfg(target_os = "macos")]
const IPV6_TCLASS: c::c_int = 61;

// TODO: Add support for old Linux versions without SOCK_CLOEXEC support
#[cfg(target_os = "linux")]
use libc::SOCK_CLOEXEC;
#[cfg(not(target_os = "linux"))]
const SOCK_CLOEXEC: c::c_int = 0;


pub struct Socket {
    fd: RawFd,
    family: c::c_int,
    peer: c::sockaddr,
    peer_len: c::socklen_t,
}

impl Socket {

    pub fn connect(addr: &SocketAddr) -> Result<Socket> {
        let family = match *addr {
            SocketAddr::V4(..) => c::AF_INET,
            SocketAddr::V6(..) => c::AF_INET6,
        };

        let fd = unsafe {
            cvt(c::socket(family, c::SOCK_RAW | SOCK_CLOEXEC, c::IPPROTO_ICMP))?
        };

        let (peer, peer_len) = addr.into_inner();

        Ok(Socket {
            fd: fd,
            family: family,
            peer: peer,
            peer_len: peer_len,
        })
    }

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

    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe {
            cvt(c::sendto(
                    self.fd,
                    buf.as_ptr() as *mut c::c_void,
                    buf.len() as c::size_t,
                    0,
                    &self.peer,
                    self.peer_len,
                )
            )?
        };

        Ok(ret as usize)
    }

    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        match self.family {
            c::AF_INET => setsockopt(self, c::IPPROTO_IP, c::IP_TTL, ttl as c::c_int),
            c::AF_INET6 => setsockopt(self, c::IPPROTO_IPV6, IPV6_UNICAST_HOPS, ttl as c::c_int),
            _ => unreachable!(),
        }
    }

    pub fn ttl(&self) -> Result<u32> {
        match self.family {
            c::AF_INET => getsockopt(self, c::IPPROTO_IP, c::IP_TTL),
            c::AF_INET6 => getsockopt(self, c::IPPROTO_IPV6, IPV6_UNICAST_HOPS),
            _ => unreachable!(),
        }
    }

    pub fn set_broadcast(&self, broadcast: bool) -> Result<()> {
        setsockopt(&self, c::SOL_SOCKET, c::SO_BROADCAST, broadcast as c::c_int)
    }

    pub fn broadcast(&self) -> Result<bool> {
        let raw: c::c_int = getsockopt(&self, c::SOL_SOCKET, c::SO_BROADCAST)?;
        Ok(raw != 0)
    }

    pub fn set_qos(&self, qos: u8) -> Result<()> {
        match self.family {
            c::AF_INET => setsockopt(&self, c::IPPROTO_IP, IP_TOS, qos as c::c_int),
            c::AF_INET6 => setsockopt(&self, c::IPPROTO_IPV6, IPV6_TCLASS, qos as c::c_int),
            _ => unreachable!(),
        }
    }

    pub fn qos(&self) -> Result<u8> {
        match self.family {
            c::AF_INET => getsockopt(&self, c::IPPROTO_IP, IP_TOS),
            c::AF_INET6 => getsockopt(&self, c::IPPROTO_IPV6, IPV6_TCLASS),
            _ => unreachable!(),
        }
    }

}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = unsafe {
            c::close(self.fd)
        };
    }
}

impl AsInner<c::c_int> for Socket {
    fn as_inner(&self) -> &c::c_int {
        &self.fd
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl IntoRawFd for Socket {
    fn into_raw_fd(self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for Socket {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let mut sockaddr: c::sockaddr = mem::zeroed();
        let res = c::getsockname(
            fd,
            &mut sockaddr,
            &mut (mem::size_of_val(&sockaddr) as c::socklen_t)
        );
        if res == -1 {
            panic!(Error::last_os_error());
        }

        Socket{
            fd: fd,
            // TODO: Probably should check for values other than `AF_INET`/`AF_INET6`
            family: sockaddr.sa_family as c::c_int,
            peer: sockaddr,
            // TODO: Proper peer_length
            peer_len: 0,
        }
    }
}
