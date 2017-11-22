use std::ffi::{CStr, NulError};
use std::fmt;
use std::result;

use failure::Fail;

use nanomsg_sys::*;

pub type Result<T> = result::Result<T, Error>;

/// An error from nanomsg
///
/// This contains the errno from the underlying nanomsg operation.
///
/// # See Also
/// * [nanomsg(7)](http://nanomsg.org/v1.1.2/nanomsg.html)
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Error(pub c_int);

impl Error {
    pub fn from_raw_nanomsg_error(errno: c_int) -> Error {
        Error(errno)
    }
}

impl From<NulError> for Error {
    fn from(_: NulError) -> Error {
        Error(EINVAL)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = unsafe { CStr::from_ptr(nn_strerror(self.0)) };
        write!(f, "Error: {}", message.to_str().unwrap())
    }
}

impl Fail for Error {}

macro_rules! error_consts {
    ($($name:ident = $cname:ident;)+) => {
        $(pub const $name: Error = Error($cname);)+
    }
}

error_consts!{
    INVALID = EINVAL;
    ADDR_FAMILY_NOT_SUPPORTED = EAFNOSUPPORT;
    MAX_FILES = EMFILE;
    TERMINATING = ETERM;
    BAD_FILE = EBADF;
    INTERRUPT = EINTR;
    NO_OPTION = ENOPROTOOPT;
    ADDR_TOO_LONG = ENAMETOOLONG;
    PROTO_NOT_SUPPORTED = EPROTONOSUPPORT;
    ADDR_NOT_AVAILABLE = EADDRNOTAVAIL;
    NO_INTERFACE = ENODEV;
    ADDR_IN_USE = EADDRINUSE;
    FAULT = EFAULT;
    NOT_SUPPORTED = ENOTSUP;
    BAD_STATE = EFSM;
    WOULD_BLOCK = EAGAIN;
    TIMED_OUT = ETIMEDOUT;
    NO_MEMORY = ENOMEM;
}
