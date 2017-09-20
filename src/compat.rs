/// Few copy-pasted things from the private Rust modules to mimic core behaviour
/// See `std::sys_common` at https://github.com/rust-lang/rust/tree/master/src/libstd/sys_common

use libc as c;
use std::u32;
use std::mem;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::convert::From;
use std::time::Duration;

use sys::Socket;

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

impl FromInner<c::sockaddr> for IpAddr {

    fn from_inner(inner: c::sockaddr) -> IpAddr {
        match inner.sa_family as i32 {
            c::AF_INET => {
                // TODO: probably `ref` can be used here
                let addr: c::sockaddr_in = unsafe {
                    *(&inner as *const _ as *const c::sockaddr_in) as c::sockaddr_in
                };
                IpAddr::V4(Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr)))
            },
            c::AF_INET6 => {
                // TODO: probably `ref` can be used here
                let addr: c::sockaddr_in6 = unsafe {
                    *(&inner as *const _ as *const c::sockaddr_in6) as c::sockaddr_in6
                };
                IpAddr::V6(Ipv6Addr::from(addr.sin6_addr.s6_addr))
            },
            _ => unreachable!(),
        }
    }

}

impl IntoInner<c::sockaddr> for IpAddr {
    fn into_inner(self) -> c::sockaddr {
        match self {
            IpAddr::V4(ref a) => {
                let ip: u32 = From::from(*a);

                let mut addr: c::sockaddr_in = unsafe { mem::zeroed() };
                addr.sin_family = c::AF_INET as c::sa_family_t;
                addr.sin_port = 0 as c::in_port_t;
                addr.sin_addr = c::in_addr {
                    s_addr: ip.to_be() as c::uint32_t
                };

                unsafe {
                    *(&addr as *const _ as *const c::sockaddr) as c::sockaddr
                }
            },
            IpAddr::V6(ref a) => {
                let mut addr: c::sockaddr_in6 = unsafe { mem::zeroed() };
                addr.sin6_family = c::AF_INET6 as c::sa_family_t;
                addr.sin6_addr = unsafe { mem::zeroed() };
                addr.sin6_addr.s6_addr = a.octets();

                unsafe {
                    *(&addr as *const _ as *const c::sockaddr) as c::sockaddr
                }
            }
        }
    }
}

pub fn setsockopt<T>(sock: &Socket, opt: c::c_int, val: c::c_int, payload: T) -> io::Result<()> {
    unsafe {
        let payload = &payload as *const T as *const c::c_void;
        cvt(c::setsockopt(*sock.as_inner(), opt, val, payload,
                          mem::size_of::<T>() as c::socklen_t))?;
        Ok(())
    }
}

pub fn getsockopt<T: Copy>(sock: &Socket, opt: c::c_int, val: c::c_int) -> io::Result<T> {
    unsafe {
        let mut slot: T = mem::zeroed();
        let mut len = mem::size_of::<T>() as c::socklen_t;
        cvt(c::getsockopt(*sock.as_inner(), opt, val,
                          &mut slot as *mut _ as *mut _,
                          &mut len))?;
        assert_eq!(len as usize, mem::size_of::<T>());
        Ok(slot)
    }
}

/// Based on the rust' `std/sys/unix/net.rs`
pub fn set_timeout(sock: &Socket, dur: Option<Duration>, kind: c::c_int) -> io::Result<()> {
    let timeout = match dur {
        Some(dur) => {
            if dur.as_secs() == 0 && dur.subsec_nanos() == 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                          "cannot set a 0 duration timeout"));
            }

            let secs = if dur.as_secs() > c::time_t::max_value() as u64 {
                c::time_t::max_value()
            } else {
                dur.as_secs() as c::time_t
            };
            let mut timeout = c::timeval {
                tv_sec: secs,
                tv_usec: (dur.subsec_nanos() / 1000) as c::suseconds_t,
            };
            if timeout.tv_sec == 0 && timeout.tv_usec == 0 {
                timeout.tv_usec = 1;
            }
            timeout
        }
        None => {
            c::timeval {
                tv_sec: 0,
                tv_usec: 0,
            }
        }
    };
    setsockopt(sock, c::SOL_SOCKET, kind, timeout)
}

/// Based on the rust' `std/sys/unix/net.rs`
pub fn timeout(sock: &Socket, kind: c::c_int) -> io::Result<Option<Duration>> {
    let raw: c::timeval = getsockopt(sock, c::SOL_SOCKET, kind)?;
    if raw.tv_sec == 0 && raw.tv_usec == 0 {
        Ok(None)
    } else {
        let sec = raw.tv_sec as u64;
        let nsec = (raw.tv_usec as u32) * 1000;
        Ok(Some(Duration::new(sec, nsec)))
    }
}