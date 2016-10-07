#![feature(question_mark)]

extern crate libc;

mod compat;
mod socket;

pub use socket::IcmpSocket;

#[cfg(test)]
mod tests;