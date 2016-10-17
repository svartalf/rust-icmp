use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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


#[test]
fn ttl() {
    let ttl = 100;

    let socket = t!(IcmpSocket::connect(ipv4()));
    t!(socket.set_ttl(ttl));

    assert_eq!(ttl, t!(socket.ttl()));
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