#![feature(question_mark)]

extern crate libc;

mod compat;
mod socket;

use socket::IcmpSocket;