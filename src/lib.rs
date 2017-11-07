extern crate nanomsg_sys;
extern crate libc;
#[macro_use]
extern crate bitflags;

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
    Pair
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
