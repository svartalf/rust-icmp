use std::io::{ErrorKind, Result};
use std::mem;
use std::net::IpAddr;

use crate::compat::{cvt, getsockopt, setsockopt, AsInner, FromInner, IntoInner};

// Following constants are not defined in libc (as for 0.2.17 version)
const IPPROTO_ICMP: libc::c_int = 1;
// Ipv4
const IP_TOS: libc::c_int = 1;
// Ipv6
const IPV6_UNICAST_HOPS: libc::c_int = 16;
const IPV6_TCLASS: libc::c_int = 67;

#[cfg(target_os = "linux")]
use libc::SOCK_CLOEXEC;
#[cfg(not(target_os = "linux"))]
const SOCK_CLOEXEC: libc::c_int = 0;

pub struct Socket {
    fd: libc::c_int,
    family: libc::c_int,
    peer: libc::sockaddr,
}

impl Socket {
    pub fn connect(addr: IpAddr) -> Result<Socket> {
        let family = match addr {
            IpAddr::V4(..) => libc::AF_INET,
            IpAddr::V6(..) => libc::AF_INET6,
        };

        let fd = unsafe { cvt(libc::socket(family, libc::SOCK_RAW | SOCK_CLOEXEC, IPPROTO_ICMP))? };

        Ok(Socket {
            fd: fd,
            family: family,
            peer: addr.into_inner(),
        })
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let ret = unsafe {
            cvt(libc::recv(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len() as libc::size_t,
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
        let mut peer: libc::sockaddr = unsafe { mem::uninitialized() };
        let ret = unsafe {
            cvt(libc::recvfrom(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len() as libc::size_t,
                0,
                &mut peer,
                &mut (mem::size_of_val(&peer) as libc::socklen_t),
            ))
        };

        match ret {
            Ok(size) => Ok((size as usize, IpAddr::from_inner(peer))),
            Err(ref err) if err.kind() == ErrorKind::Interrupted => Ok((0, IpAddr::from_inner(peer))),
            Err(err) => Err(err),
        }
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = unsafe {
            cvt(libc::sendto(
                self.fd,
                buf.as_ptr() as *mut libc::c_void,
                buf.len() as libc::size_t,
                0,
                &self.peer,
                mem::size_of_val(&self.peer) as libc::socklen_t,
            ))?
        };

        Ok(ret as usize)
    }

    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        match self.family {
            libc::AF_INET => setsockopt(self, libc::IPPROTO_IP, libc::IP_TTL, ttl as libc::c_int),
            libc::AF_INET6 => setsockopt(self, libc::IPPROTO_IPV6, IPV6_UNICAST_HOPS, ttl as libc::c_int),
            _ => unreachable!(),
        }
    }

    pub fn ttl(&self) -> Result<u32> {
        match self.family {
            libc::AF_INET => getsockopt(self, libc::IPPROTO_IP, libc::IP_TTL),
            libc::AF_INET6 => getsockopt(self, libc::IPPROTO_IPV6, IPV6_UNICAST_HOPS),
            _ => unreachable!(),
        }
    }

    pub fn set_broadcast(&self, broadcast: bool) -> Result<()> {
        setsockopt(&self, libc::SOL_SOCKET, libc::SO_BROADCAST, broadcast as libc::c_int)
    }

    pub fn broadcast(&self) -> Result<bool> {
        let raw: libc::c_int = getsockopt(&self, libc::SOL_SOCKET, libc::SO_BROADCAST)?;
        Ok(raw != 0)
    }

    pub fn set_qos(&self, qos: u8) -> Result<()> {
        match self.family {
            libc::AF_INET => setsockopt(&self, libc::IPPROTO_IP, IP_TOS, qos as libc::c_int),
            libc::AF_INET6 => setsockopt(&self, libc::IPPROTO_IPV6, IPV6_TCLASS, qos as libc::c_int),
            _ => unreachable!(),
        }
    }

    pub fn qos(&self) -> Result<u8> {
        match self.family {
            libc::AF_INET => getsockopt(&self, libc::IPPROTO_IP, IP_TOS),
            libc::AF_INET6 => getsockopt(&self, libc::IPPROTO_IPV6, IPV6_TCLASS),
            _ => unreachable!(),
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = unsafe { libc::close(self.fd) };
    }
}

impl AsInner<libc::c_int> for Socket {
    fn as_inner(&self) -> &libc::c_int {
        &self.fd
    }
}
