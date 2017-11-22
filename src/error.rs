//! Nanomsg errors
//!
//! This contains an [`Error`](struct.Error.html) type for errors encountered
//! from the nanomsg C interface, as well as a convenience [`Result`](type.Result.html)
//! using that Error type.
//!
//! It also contains constants for specific errors, including all errors described in the nanomsg
//! documentation (as of 2017-11-22).
use std::ffi::{CStr, NulError};
use std::fmt;
use std::result;

use failure::Fail;

use nanomsg_sys::*;

/// Specialized [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html) type for nanomsg
/// errors.
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
    ($($(#[$attrs:meta])* $name:ident = $cname:ident;)+) => {
        $($(#[$attrs])* pub const $name: Error = Error($cname);)+
    }
}

error_consts!{
    /// Invalid argument
    INVALID = EINVAL;
    /// The address family isn't supported
    ADDR_FAMILY_NOT_SUPPORTED = EAFNOSUPPORT;
    /// The maximum number of file descriptors has been reached
    MAX_FILES = EMFILE;
    /// Attempting to use a socket, after `nn_term` has been called
    TERMINATING = ETERM;
    /// The file descriptor for the socket isn't valid
    BAD_FILE = EBADF;
    /// The operation was interrupted by a signal before it completed
    INTERRUPT = EINTR;
    /// The specified socket option doesn't exist for this socket.
    NO_OPTION = ENOPROTOOPT;
    /// The specified address exceeds the maximum address length
    ADDR_TOO_LONG = ENAMETOOLONG;
    /// The protocol isn't supported or is unrecognized
    PROTO_NOT_SUPPORTED = EPROTONOSUPPORT;
    /// The address for a bind is not local
    ADDR_NOT_AVAILABLE = EADDRNOTAVAIL;
    /// The address specifies a non-existent interface
    NO_INTERFACE = ENODEV;
    /// The requested address is already in use
    ADDR_IN_USE = EADDRINUSE;
    /// Segmentation Fault (for example, attempting to read or write to null)
    FAULT = EFAULT;
    /// The operation isn't supported
    NOT_SUPPORTED = ENOTSUP;
    /// Attempt to perform an operations while the socket is in an incompatible state.
    ///
    /// For example, trying to receive when the socket expects a send, or vice versa.
    BAD_STATE = EFSM;
    /// The operation would block if the call was in blocking mode.
    WOULD_BLOCK = EAGAIN;
    /// The operaton timed out.
    TIMED_OUT = ETIMEDOUT;
    /// Unable to allocate due to insufficient memory.
    NO_MEMORY = ENOMEM;
}
