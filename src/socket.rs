use std::ffi::CString;
use error::{Error, Result};
use std::mem;
use std::ptr;

use nanomsg_sys::*;

use alloc::MessageBuffer;

pub type RawFd = c_int;

macro_rules! error_guard {
    ($ret:expr) => {
        if $ret == -1 {
            return Err(last_error())
        }
    }
}

bitflags!{
    #[derive(Default)]
    pub struct Flags: c_int {
        const DONTWAIT = NN_DONTWAIT;
    }
}

pub struct Endpoint {
    socket: c_int,
    id: c_int
}

impl Endpoint {
    pub fn shutdown(self) -> Result<()> {
        unsafe {
            error_guard!(nn_shutdown(self.socket, self.id));
            Ok(())
        }
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Domain {
    SP = AF_SP,
    SPRaw = AF_SP_RAW
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Protocol {
    Pub = NN_PUB,
    Sub = NN_SUB,
    Bus = NN_BUS,
    Req = NN_REQ,
    Rep = NN_REP,
    Push = NN_PUSH,
    Pull = NN_PULL,
    Surveyor = NN_SURVEYOR,
    Respondent = NN_RESPONDENT,
    Pair = NN_PAIR
}

pub struct Socket(RawFd);

impl Socket {

    pub fn new(domain: Domain, protocol: Protocol) -> Result<Socket> {
        let fd = unsafe {
            nn_socket(domain as c_int, protocol as c_int)
        };
        error_guard!(fd);
        Ok(Socket(fd))
    }

    pub fn bind(&self, addr: &str) -> Result<Endpoint> {
        unsafe {
            let c_addr = CString::new(addr)?;
            let endpoint = nn_bind(self.0, c_addr.as_ptr());
            error_guard!(endpoint);
            Ok(Endpoint {
                socket: self.0,
                id: endpoint
            })
        }
    }

    pub fn connect(&self, addr: &str) -> Result<Endpoint> {
        unsafe {
            let c_addr = CString::new(addr)?;
            let endpoint = nn_connect(self.0, c_addr.as_ptr());
            error_guard!(endpoint);
            Ok(Endpoint {
                socket: self.0,
                id: endpoint
            })
        }
    }

    pub fn recv(&self, flags: Flags) -> Result<MessageBuffer> {
        let mut buffer: *mut c_void = ptr::null_mut();
        let size = unsafe {
            nn_recv(self.0, &mut buffer as *mut _ as *mut c_void, NN_MSG, flags.bits)
        };
        error_guard!(size);
        Ok(unsafe {
            MessageBuffer::from_raw(buffer, size as usize)
        })
    }

    pub fn recv_buf(&self, buffer: &mut [u8], flags: Flags) -> Result<usize> {
        let size = unsafe {
            nn_recv(self.0, buffer.as_mut_ptr() as *mut c_void, buffer.len(), flags.bits)
        };
        error_guard!(size);
        Ok(size as usize)
    }

    pub fn send_buf(&self, buffer: &[u8], flags: Flags) -> Result<usize> {
        let size = unsafe {
            nn_send(self.0, buffer.as_ptr() as *const c_void, buffer.len(), flags.bits)
        };
        error_guard!(size);
        Ok(size as usize)
    }

    pub fn send(&self, buffer: MessageBuffer, flags: Flags) -> Result<usize> {
        let size = unsafe {
            let buf_ptr = buffer.into_raw();
            nn_send(self.0, &buf_ptr as *const _ as *const c_void, NN_MSG, flags.bits)
        };
        error_guard!(size);
        Ok(size as usize)
    }

    pub fn device(sock1: &Socket, sock2: &Socket) -> Error {
        unsafe { nn_device(sock1.0, sock2.0) };
        // if nn_device returns there was an error
        last_error()
    }

    pub fn loopback_device(sock: &Socket) -> Error {
        unsafe { nn_device(sock.0, -1) };
        // if nn_device returns there was an error
        last_error()
    }

    pub fn terminate() {
        unsafe { nn_term() };
    }

    #[inline]
    pub unsafe fn as_raw_fd(&self) -> RawFd {
        self.0
    }

    #[inline]
    pub unsafe fn into_raw_fd(self) -> RawFd {
        let fd = self.0;
        mem::forget(self);
        fd
    }

    #[inline]
    pub unsafe fn from_raw(fd: RawFd) -> Socket {
        Socket(fd)
    }

    pub unsafe fn set_option<T: OptionSet>(&self, level: c_int, option: c_int, value: T) -> Result<()> {
        T::set(self.0, level, option, value)
    }

    pub unsafe fn get_option<T: OptionGet>(&self, level: c_int, option: c_int) -> Result<T> {
        T::get(self.0, level, option)
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            nn_close(self.0);
        }
    }
}

pub trait OptionGet: Sized {
    fn get(fd: c_int, level: c_int, option: c_int) -> Result<Self>;
}
pub trait OptionSet: Sized {
    fn set(fd: c_int, level: c_int, option: c_int, val: Self) -> Result<()>;
}

impl OptionGet for i32 {
    fn get(fd: c_int, level: c_int, option: c_int) -> Result<i32> {
        let mut value: i32 = 0;
        let mut size = mem::size_of::<i32>();
        let ret = unsafe {
            nn_getsockopt(fd,
                          level,
                          option,
                          &mut value as *mut _ as *mut c_void,
                          &mut size as *mut _)
        };
        error_guard!(ret);
        Ok(value)
    }
}
impl OptionSet for i32 {
    fn set(fd: c_int, level: c_int, option: c_int, val: i32) -> Result<()> {
        let ret = unsafe {
            nn_setsockopt(fd,
                          level,
                          option,
                          &val as *const _ as *const c_void,
                          mem::size_of::<i32>())
        };
        error_guard!(ret);
        Ok(())
    }
}

impl OptionGet for bool {
    fn get(fd: c_int, level: c_int, option: c_int) -> Result<bool> {
        <i32 as OptionGet>::get(fd, level, option).map(|v| v != 0)
    }
}
impl OptionSet for bool {
    fn set(fd: c_int, level: c_int, option: c_int, val: bool) -> Result<()> {
        <i32 as OptionSet>::set(fd, level, option, val as i32)
    }
}

fn last_error() -> Error {
    let errno = unsafe { nn_errno() };
    Error::from_raw_nanomsg_error(errno)
}
