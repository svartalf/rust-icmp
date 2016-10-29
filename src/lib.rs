#[deny(missing_docs)]

extern crate libc;

#[doc(hidden)]
mod compat;
mod socket;

#[cfg(unix)]
#[path = "sys/unix.rs"] mod sys;

#[cfg(windows)]
#[path = "sys/mod.rs"] mod sys;

pub use socket::IcmpSocket;

#[cfg(test)]
mod tests;