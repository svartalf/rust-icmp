
use std::net::IpAddr;
use std::io::{Result, ErrorKind};
use std::mem;

use libc as c;

use compat::{IntoInner, FromInner, cvt};

const IPPROTO_ICMP: c::c_int = 1;


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
            cvt(c::socket(family, c::SOCK_RAW, IPPROTO_ICMP))?
        };

        Ok(IcmpSocket {
            fd: fd,
            peer: addr.into_inner(),
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
                    mem::size_of_val(&self.peer) as c::socklen_t,
                )
            )?
        };

        Ok(ret as usize)
    }

    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        let payload = &ttl as *const u32 as *const c::c_void;
        unsafe {
            cvt(c::setsockopt(self.fd, c::IPPROTO_IP, c::IP_TTL, payload, mem::size_of::<u32>() as c::socklen_t))?
        };

        Ok(())
    }

    pub fn ttl(&self) -> Result<u32> {
        unsafe {
            let mut slot: u32 = mem::zeroed();
            let mut len = mem::size_of::<u32>() as c::socklen_t;
            cvt(c::getsockopt(self.fd, c::IPPROTO_IP, c::IP_TTL,
                &mut slot as *mut _ as *mut _, &mut len))?;

            Ok(slot)
        }

    }

}

impl Drop for IcmpSocket {
    fn drop(&mut self) {
        let _ = unsafe {
            c::close(self.fd)
        };
    }
}
