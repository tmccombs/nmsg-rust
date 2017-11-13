use std::ffi::CString;
use error::{Error, Result};
use std::mem;
use std::ptr;
#[cfg(windows)]
use std::os::windows::io::RawSocket;

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

/// A request for polling a socket and the poll result
#[derive(Copy, Clone)]
pub struct Poll(nn_pollfd);

impl Poll {
    /// Return if the socket is available for receiving a message
    #[inline]
    pub fn can_receive(&self) -> bool {
        self.0.pollin_result()
    }

    /// Return if the socket is available to send a message
    #[inline]
    pub fn can_send(&self) -> bool {
        self.0.pollout_result()
    }
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

    pub fn make_poll(&self, pollin: bool, pollout: bool) -> Poll {
        Poll(nn_pollfd::new(self.0, pollin, pollout))
    }

    /// Checks if it's possible to send or receive messages without blocking on set of sockets.
    ///
    /// # Arguments
    ///
    /// * polls - An array of `Poll` objects created with `Socket::make_poll` for the sockets to
    /// poll. If a socket is ready for send or receive operations, the corresponding `Poll` object
    /// will be updated to report the available operations.
    /// * timeout - How long (in milliseconds) the poll function should block if there are no
    /// events to report.
    ///
    /// # Returns
    ///
    /// How many sockets are available to send and/or receive, or an error.
    pub fn poll(polls: &mut [Poll], timeout: i32) -> Result<usize> {
        let nready = unsafe { nn_poll(polls.as_mut_ptr() as *mut nn_pollfd, polls.len() as c_int, timeout) };
        error_guard!(nready);
        Ok(nready as usize)
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

impl OptionGet for Vec<u8> {
    fn get(fd: c_int, level: c_int, option: c_int) -> Result<Vec<u8>> {
        const BUF_SIZE: usize = 128;
        let mut value = vec![0; BUF_SIZE];
        let mut size = BUF_SIZE;
        let ret = unsafe {
            nn_getsockopt(fd,
                          level,
                          option,
                          value.as_mut_ptr() as *mut c_void,
                          &mut size as *mut _)
        };
        error_guard!(ret);
        value.truncate(size);
        Ok(value)
    }
}

impl<'a> OptionSet for &'a [u8] {
    fn set(fd: c_int, level: c_int, option: c_int, val: &[u8]) -> Result<()> {
        let ret = unsafe {
            nn_setsockopt(fd,
                          level,
                          option,
                          val.as_ptr() as *const c_void,
                          val.len())
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

#[cfg(windows)]
impl OptionGet for RawSocket {
    fn get(fd: c_int, level: c_int, option: c_int) -> Result<RawSocket> {
        let mut value: RawSocket = 0;
        let mut size = mem::size_of::<RawSocket>();
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

fn last_error() -> Error {
    let errno = unsafe { nn_errno() };
    Error::from_raw_nanomsg_error(errno)
}
