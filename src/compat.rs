/// Few copy-pasted things from the private Rust modules to mimic core behaviour
/// See `std::sys_common` at https://github.com/rust-lang/rust/tree/master/src/libstd/sys/common

use std::u32;
use std::mem;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::convert::From;
use std::time::Duration;

use crate::sys::Socket;

pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

pub fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

#[doc(hidden)]
pub trait AsInner<Inner: ?Sized> {
    fn as_inner(&self) -> &Inner;
}

#[doc(hidden)]
pub trait FromInner<Inner> {
    fn from_inner(inner: Inner) -> Self;
}

#[doc(hidden)]
pub trait IntoInner<Inner> {
    fn into_inner(self) -> Inner;
}

impl FromInner<libc::sockaddr> for IpAddr {

    fn from_inner(inner: libc::sockaddr) -> IpAddr {
        match inner.sa_family as i32 {
            libc::AF_INET => {
                // TODO: probably `ref` can be used here
                let addr: libc::sockaddr_in = unsafe {
                    *(&inner as *const _ as *const libc::sockaddr_in) as libc::sockaddr_in
                };
                IpAddr::V4(Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr)))
            },
            libc::AF_INET6 => {
                // TODO: probably `ref` can be used here
                let addr: libc::sockaddr_in6 = unsafe {
                    *(&inner as *const _ as *const libc::sockaddr_in6) as libc::sockaddr_in6
                };
                IpAddr::V6(Ipv6Addr::from(addr.sin6_addr.s6_addr))
            },
            _ => unreachable!(),
        }
    }

}

impl IntoInner<libc::sockaddr> for IpAddr {
    fn into_inner(self) -> libc::sockaddr {
        match self {
            IpAddr::V4(ref a) => {
                let ip: u32 = From::from(*a);

                let mut addr: libc::sockaddr_in = unsafe { mem::zeroed() };
                addr.sin_family = libc::AF_INET as libc::sa_family_t;
                addr.sin_port = 0 as libc::in_port_t;
                addr.sin_addr = libc::in_addr {
                    s_addr: ip.to_be() as libc::uint32_t
                };

                unsafe {
                    *(&addr as *const _ as *const libc::sockaddr) as libc::sockaddr
                }
            },
            IpAddr::V6(ref a) => {
                let mut addr: libc::sockaddr_in6 = unsafe { mem::zeroed() };
                addr.sin6_family = libc::AF_INET6 as libc::sa_family_t;
                addr.sin6_addr = unsafe { mem::zeroed() };
                addr.sin6_addr.s6_addr = a.octets();

                unsafe {
                    *(&addr as *const _ as *const libc::sockaddr) as libc::sockaddr
                }
            }
        }
    }
}

pub fn setsockopt<T>(sock: &Socket, opt: libc::c_int, val: libc::c_int, payload: T) -> io::Result<()> {
    unsafe {
        let payload = &payload as *const T as *const libc::c_void;
        cvt(libc::setsockopt(*sock.as_inner(), opt, val, payload,
                          mem::size_of::<T>() as libc::socklen_t))?;
        Ok(())
    }
}

pub fn getsockopt<T: Copy>(sock: &Socket, opt: libc::c_int, val: libc::c_int) -> io::Result<T> {
    unsafe {
        let mut slot: T = mem::zeroed();
        let mut len = mem::size_of::<T>() as libc::socklen_t;
        cvt(libc::getsockopt(*sock.as_inner(), opt, val,
                          &mut slot as *mut _ as *mut _,
                          &mut len))?;
        assert_eq!(len as usize, mem::size_of::<T>());
        Ok(slot)
    }
}

/// Based on the rust' `std/sys/unix/net.rs`
pub fn set_timeout(sock: &Socket, dur: Option<Duration>, kind: libc::c_int) -> io::Result<()> {
    let timeout = match dur {
        Some(dur) => {
            if dur.as_secs() == 0 && dur.subsec_nanos() == 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                          "cannot set a 0 duration timeout"));
            }

            let secs = if dur.as_secs() > libc::time_t::max_value() as u64 {
                libc::time_t::max_value()
            } else {
                dur.as_secs() as libc::time_t
            };
            let mut timeout = libc::timeval {
                tv_sec: secs,
                tv_usec: (dur.subsec_nanos() / 1000) as libc::suseconds_t,
            };
            if timeout.tv_sec == 0 && timeout.tv_usec == 0 {
                timeout.tv_usec = 1;
            }
            timeout
        }
        None => {
            libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            }
        }
    };
    setsockopt(sock, libc::SOL_SOCKET, kind, timeout)
}

/// Based on the rust' `std/sys/unix/net.rs`
pub fn timeout(sock: &Socket, kind: libc::c_int) -> io::Result<Option<Duration>> {
    let raw: libc::timeval = getsockopt(sock, libc::SOL_SOCKET, kind)?;
    if raw.tv_sec == 0 && raw.tv_usec == 0 {
        Ok(None)
    } else {
        let sec = raw.tv_sec as u64;
        let nsec = (raw.tv_usec as u32) * 1000;
        Ok(Some(Duration::new(sec, nsec)))
    }
}
