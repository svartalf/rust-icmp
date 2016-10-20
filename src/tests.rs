use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use IcmpSocket;

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
        }
    }
}

fn ipv4() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}

fn ipv6() -> IpAddr {
    IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
}


#[test]
fn ttl_v4() {
    let ttl = 100;

    let socket = t!(IcmpSocket::connect(ipv4()));
    t!(socket.set_ttl(ttl));

    assert_eq!(ttl, t!(socket.ttl()));
}

#[test]
fn ttl_v6() {
    let ttl = 100;

    let socket = t!(IcmpSocket::connect(ipv6()));
    t!(socket.set_ttl(ttl));

    assert_eq!(ttl, t!(socket.ttl()));
}

#[test]
fn read_timeout() {
    let timeout = Duration::new(2, 0);
    let socket = t!(IcmpSocket::connect(ipv4()));

    t!(socket.set_read_timeout(Some(timeout)));
    assert_eq!(Some(timeout), t!(socket.read_timeout()));

    t!(socket.set_read_timeout(None));
    assert_eq!(None, t!(socket.read_timeout()));
}

#[test]
fn write_timeout() {
    let timeout = Duration::new(2, 0);
    let socket = t!(IcmpSocket::connect(ipv4()));

    t!(socket.set_write_timeout(Some(timeout)));
    assert_eq!(Some(timeout), t!(socket.write_timeout()));

    t!(socket.set_write_timeout(None));
    assert_eq!(None, t!(socket.write_timeout()));
}

#[test]
fn broadcast_on() {
    let socket = t!(IcmpSocket::connect(ipv4()));
    t!(socket.set_broadcast(true));

    assert_eq!(true, t!(socket.broadcast()));
}

#[test]
fn broadcast_off() {
    let socket = t!(IcmpSocket::connect(ipv4()));
    // Enabling broadcast by default to ensure value will change
    t!(socket.set_broadcast(true));

    t!(socket.set_broadcast(false));
    assert_eq!(false, t!(socket.broadcast()));
}