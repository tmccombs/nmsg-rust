#![warn(missing_docs)]
//! This is a rust library for [Nanomsg](http://nanomsg.org/index.html).
//!
//! It is a wrapper around the C API that provides a more rusty interface.
//!
//! In particular this library provides:
//! * A [`MessageBuffer`](alloc/struct.MessageBuffer.html) type to wrap zero-copy memory allocations in a `Vec`-like structure.
//! * A seperate struct for each type of Nanomsg socket, which only implement methods for supported
//! operations (for example there is now receive method for the Pub socket).
//! * Helper methods for multi-step operations, like sending a request and waiting for a reply or
//! waiting for and replying to a request.
//!
//! # See Also
//! * [Nanomsg](http://nanomsg.org/index.html)
//! * [nanomsg(7)](http://nanomsg.org/v1.1.2/nanomsg.html)
//! * [nanomsg-sys](https://docs.rs/nanomsg-sys/0.6.2/nanomsg_sys/)

extern crate nanomsg_sys;
extern crate libc;
#[macro_use]
extern crate bitflags;
extern crate failure;

pub mod alloc;
pub mod error;
pub mod socket;
pub mod protocol;

pub use alloc::{MessageBuffer};
pub use error::{Error, Result};
pub use protocol::{
    Pub, Sub,
    Bus,
    Req, Rep,
    Push, Pull,
    Surveyor, Respondent,
    Pair,

    SPSocket, SPRecv, SPSend, Loopback
};
