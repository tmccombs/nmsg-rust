use std::ffi::{CStr, NulError};
use std::fmt;
use std::result;
use std::string::FromUtf8Error;

use nng_sys::*;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    Nng(i32),
    NulByte(#[cause] NulError),
    Utf8Error(#[cause] FromUtf8Error)
}

use self::Error::*;

impl Error {
    pub fn from_raw(errno: i32) -> Error {
        Error::Nng(errno)
    }
}

impl From<NulError> for Error {
    fn from(e: NulError) -> Error {
        NulByte(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Error {
        Utf8Error(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Nng(errno) => {
                let message = unsafe { CStr::from_ptr(nng_strerror(errno)) };
                write!(f, "{}", message.to_str().unwrap())
            }
            NulByte(ref e) => e.fmt(f),
            Utf8Error(ref e) => e.fmt(f)
        }
    }
}

#[macro_export]
macro_rules! error_consts {
    ($($(#[$attrs:meta])* $name:ident = $cname:ident;)+) => {
        $($(#[$attrs])* pub const $name: Error = Error::Nng($cname);)+
    }
}

error_consts! {
    INTERRUPT = NNG_EINTR;
    OUT_OF_MEMORY = NNG_ENOMEM;
    INVALID = NNG_EINVAL;
    BUSY = NNG_EBUSY;
    TIMED_OUT = NNG_ETIMEDOUT;
    CONNECTION_REFUSED = NNG_ECONNREFUSED;
    CLOSED = NNG_ECLOSED;
    TRY_AGAIN = NNG_EAGAIN;
    NOT_SUPPORTED = NNG_ENOTSUP;
    ADDRESS_IN_USE = NNG_EADDRINUSE;
    BAD_STATE = NNG_ESTATE;
    NO_ENTRY = NNG_ENOENT;
    PROTOCOL_ERROR = NNG_EPROTO;
    UNREACHABLE = NNG_EUNREACHABLE;
    INVALID_ADDRESS = NNG_EADDRINVAL;
    PERMISSION_DENIED = NNG_EPERM;
    MSG_TOO_LARGE = NNG_EMSGSIZE;
    CONNECTION_ABORTED = NNG_ECONNABORTED;
    CONNECTION_RESET = NNG_ECONNRESET;
    CANCELED = NNG_ECANCELED;
    MAX_FILES = NNG_ENOFILES;
    NO_SPACE = NNG_ENOSPC;
    EXISTS = NNG_EEXIST;
    READONLY = NNG_EREADONLY;
    WRITEONLY = NNG_EWRITEONLY;
    INTERNAL_ERROR = NNG_EINTERNAL;
    SYSTEM_ERROR = NNG_ESYSERR;
    TRANSPORT_ERROR = NNG_ETRANERR;
}

macro_rules! error_guard {
    ($ret:expr) => {
        let r = $ret;
        if r != 0 {
            return Err($crate::error::Error::from_raw(r));
        }
    }
}


