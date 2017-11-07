use std::error;
use std::ffi::{CStr, NulError};
use std::fmt;
use std::result;

use nanomsg_sys::*;

pub type Result<T> = result::Result<T, Error>;

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

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.0 {
            EINVAL => "Invalid argument",
            EAFNOSUPPORT => "Address family is not supported",
            EMFILE => "Maximum number of file descriptors exceeded",
            ETERM => "Process is terminating",
            EBADF => "Bad file descriptor",
            EINTR => "Interupt",
            ENOPROTOOPT => "No such option",
            ENAMETOOLONG => "Address is too long",
            EPROTONOSUPPORT => "Protocol is not supported",
            EADDRNOTAVAIL => "Address is not available",
            ENODEV => "No such interface",
            EADDRINUSE => "Address is already in use",
            EFAULT => "Memory fault",
            ENOTSUP => "Operation not supported",
            EFSM => "Operation not supported in current state",
            EAGAIN => "Operation would block",
            ETIMEDOUT => "Operation timed out",
            ENOMEM => "Out of Memory",
            _ => "Unknown error from nanomsg"
        }
    }
}

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
