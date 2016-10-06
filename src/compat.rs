/// Few copy-pasted things from the private Rust modules to mimic core behaviour
/// See `std::sys_common` at https://github.com/rust-lang/rust/tree/master/src/libstd/sys/common

use libc as c;
use std::u32;
use std::mem;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::convert::From;

#[doc(hidden)]
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
            _ => panic!("Totally unknown AF family. I'm out.")
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
                addr.sin6_family = c::AF_INET6 as u16;
                addr.sin6_addr = unsafe { mem::zeroed() };
                addr.sin6_addr.s6_addr = a.octets();

                unsafe {
                    *(&addr as *const _ as *const c::sockaddr) as c::sockaddr
                }
            }
        }
    }
}