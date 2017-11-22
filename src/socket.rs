//! Lower level interface for nanomsg sockets.
//!
//! This is a thin wrapper around the C API
//! and is primarily used to implement the traits in [`protocols`](../protocol/index.html).
use std::ffi::CString;
use std::mem;
use std::ptr;
#[cfg(windows)]
use std::os::windows::io::RawSocket;

use nanomsg_sys::*;

use alloc::MessageBuffer;
use error::{Error, Result};

/// A raw SP Socket File Descriptor
pub type RawFd = c_int;

macro_rules! error_guard {
    ($ret:expr) => {
        if $ret == -1 {
            return Err(last_error())
        }
    }
}

bitflags!{
    /// Flags to use when sending and receiving messages.
    #[derive(Default)]
    pub struct Flags: c_int {
        /// Don't block for the operation.
        ///
        /// If this is specified the operation should be done
        /// in non-blocking mode.
        const DONTWAIT = NN_DONTWAIT;
    }
}

/// A handle to an endpoint created with `bind` or `connect`.
pub struct Endpoint {
    socket: c_int,
    id: c_int
}

impl Endpoint {
    /// Shut down the endpoint.
    ///
    /// This closes a listener or connection that was previously
    /// established.
    pub fn shutdown(self) -> Result<()> {
        unsafe {
            error_guard!(nn_shutdown(self.socket, self.id));
            Ok(())
        }
    }
}

/// The address family domain of a socket
#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Domain {
    /// Normal Scalability Protocol domain
    SP = AF_SP,
    /// Raw Scalability Protocol domain
    ///
    /// This is primarily used for creating
    /// devices.
    ///
    /// # See Also
    ///
    /// * [`Socket::device`](../struct.Socket.html#method.device)
    /// * [`SPSocket::device`](../../protocol/trait.SPSocket.html#method.device)
    /// * [nn_socket(3)](http:///nanomsg.org/v1.1.2/nn_socket.html)
    SPRaw = AF_SP_RAW
}

/// The protocol of a socket
#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Protocol {
    /// Publish (PubSub)
    ///
    /// See [nn_pubsub(7)](http://nanomsg.org/v1.1.2/nn_pubsub.html)
    Pub = NN_PUB,
    /// Subscribe (PubSub)
    ///
    /// See [nn_pubsub(7)](http://nanomsg.org/v1.1.2/nn_pubsub.html)
    Sub = NN_SUB,
    /// Bus
    ///
    /// See [nn_bus(7)](http://nanomsg.org/v1.1.2/nn_pubsub.html)
    Bus = NN_BUS,
    /// Request (ReqRep)
    ///
    /// See [nn_reqrep(7)](http://nanomsg.org/v1.1.2/nn_reqrep.html)
    Req = NN_REQ,
    /// Reply (ReqRep)
    ///
    /// See [nn_reqrep(7)](http://nanomsg.org/v1.1.2/nn_reqrep.html)
    Rep = NN_REP,
    /// Push (Pipeline)
    ///
    /// See [nn_pipeline(7)](http://nanomsg.org/v1.1.2/nn_pipeline.html)
    Push = NN_PUSH,
    /// Pull (Pipeline)
    ///
    /// See [nn_pipeline(7)](http://nanomsg.org/v1.1.2/nn_pipeline.html)
    Pull = NN_PULL,
    /// Surveyor (Survey)
    ///
    /// See [nn_survey(7)](http://nanomsg.org/v1.1.2/nn_survey.html)
    Surveyor = NN_SURVEYOR,
    /// Respondent (Survey)
    ///
    /// See [nn_survey(7)](http://nanomsg.org/v1.1.2/nn_survey.html)
    Respondent = NN_RESPONDENT,
    /// Pair
    ///
    /// See [nn_pair(7)](http://nanomsg.org/v1.1.2/nn_pair.html)
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

/// Wrapper type around a nanomsg socket.
///
/// This is a lower level interface.
/// In most cases you probably want to use one of the
/// [`SPSocket`](../../trait.SPSocket.html) instances.
pub struct Socket(RawFd);

impl Socket {

    /// Create a new socket
    ///
    /// # Arguments
    ///
    /// * `domain`: The address family domain. Typically this will be `Domain::SP`, unless
    /// this will be used for creating a device.
    /// * `protocol`: The scalability protocol to use.
    ///
    /// # See Also
    /// * [`Protocol`](../enum.Protocol.html)
    /// * [nn_socket](http://nanomsg.org/v1.1.2/nn_socket.html)
    pub fn new(domain: Domain, protocol: Protocol) -> Result<Socket> {
        let fd = unsafe {
            nn_socket(domain as c_int, protocol as c_int)
        };
        error_guard!(fd);
        Ok(Socket(fd))
    }

    /// Bind the socket to an address.
    ///
    /// This binds this socket to an address and listens for incoming connections.
    ///
    /// A single socket can be bound and/or connected to multiple addresses.
    ///
    /// # Arguments
    ///
    /// `addr` is the (local) address to bind to. See [nanomsg
    /// manual](http://nanomsg.org/v1.1.2/nanomsg.html) for format.
    ///
    /// # Returns
    ///
    /// An `Endpoint` which can be used to stop listening on the
    /// bound address.
    ///
    /// # See Also
    ///
    /// * [nn_bind](http://nanomsg.org/v1.1.2/nn_bind.html)
    /// * [`connect`](#method.connect)
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

    /// Connect the socket to an address.
    ///
    /// This connects this socket to a remote socket that is listening on the
    /// specified address.
    ///
    /// A single socket can be bound and/or connected to multiple addresses.
    ///
    /// # Arguments
    ///
    /// `addr` is the (remote) address to connect to. See [nanomsg
    /// manual](http://nanomsg.org/v1.1.2/nanomsg.html) for format.
    ///
    /// # Returns
    ///
    /// An `Endpoint` which can be used to close the connection
    ///
    /// # See Also
    ///
    /// * [nn_connect](http://nanomsg.org/v1.1.2/nn_connect.html)
    /// * [`bind`](#method.bind)
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

    /// Receive a message.
    ///
    /// # See Also
    /// * [`SPRecv`](../../trait.SPRecv)
    /// * [nn_recv(3)](http://nanomsg.org/v1.1.2/nn_recv.html)
    /// * [`recv_buf`](#method.recv_buf)
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

    /// Receive a message into an existing buffer.
    ///
    /// # Returns
    ///
    /// The number of bytes in the message received.
    ///
    /// # See Also
    /// * [`SPRecv`](../../trait.SPRecv.html)
    /// * [nn_recv(3)](http://nanomsg.org/v1.1.2/nn_recv.html)
    /// * [`recv`](#method.recv)
    pub fn recv_buf(&self, buffer: &mut [u8], flags: Flags) -> Result<usize> {
        let size = unsafe {
            nn_recv(self.0, buffer.as_mut_ptr() as *mut c_void, buffer.len(), flags.bits)
        };
        error_guard!(size);
        Ok(size as usize)
    }


    /// Send a message.
    ///
    /// # See Also
    /// * [`SPSend`](../../trait.SPSend.html)
    /// * [nn_send(3)](http://nanomsg.org/v1.1.2/nn_send.html)
    /// * [`send_buf`](#method.send_buf)
    pub fn send(&self, buffer: MessageBuffer, flags: Flags) -> Result<usize> {
        let size = unsafe {
            let buf_ptr = buffer.into_raw();
            nn_send(self.0, &buf_ptr as *const _ as *const c_void, NN_MSG, flags.bits)
        };
        error_guard!(size);
        Ok(size as usize)
    }

    /// Send a message from a slice.
    ///
    /// # See Also
    /// * [`SPSend`](../../trait.SPSend.html)
    /// * [nn_send(3)](http://nanomsg.org/v1.1.2/nn_send.html)
    /// * [`send`](#method.send)
    pub fn send_buf(&self, buffer: &[u8], flags: Flags) -> Result<usize> {
        let size = unsafe {
            nn_send(self.0, buffer.as_ptr() as *const c_void, buffer.len(), flags.bits)
        };
        error_guard!(size);
        Ok(size as usize)
    }

    /// Starts a device to forward messages between two sockets.
    ///
    /// # See Also
    /// * [nn_device(3)](http://nanomsg.org/v1.1.2/nn_device.html)
    /// * [`Socket::device`](../../trait.SPSocket.html#method.device)
    /// * [`loopback_device`](#method.loopback_device)
    /// * [`terminate`](#method.terminate)
    pub fn device(sock1: &Socket, sock2: &Socket) -> Error {
        unsafe { nn_device(sock1.0, sock2.0) };
        // if nn_device returns there was an error
        last_error()
    }

    /// Starts a device to forward messages between two sockets.
    ///
    /// # See Also
    /// * [nn_device(3)](http://nanomsg.org/v1.1.2/nn_device.html)
    /// * [`Loopback`](../../trait.Loopback.html)
    /// * [`loopback`](#method.loopback)
    /// * [`terminate`](#method.terminate)
    pub fn loopback_device(sock: &Socket) -> Error {
        unsafe { nn_device(sock.0, -1) };
        // if nn_device returns there was an error
        last_error()
    }

    /// Notify all sockets about process termination.
    ///
    /// Let all sockets know that the process is about to terminate so that they
    /// can be closed and resources freed.
    ///
    /// # See Also
    /// * [nn_term(3)](http://nanomsg.org/v1.1.2/nn_term.html)
    pub fn terminate() {
        unsafe { nn_term() };
    }

    /// Create nanomsg poll file descriptor.
    ///
    /// This can be used with [`poll`](#method.poll) to poll if
    /// the socket is available.
    ///
    /// # Arguments
    /// * `pollin`: If true polls will check if the socket can receive a message without blocking.
    /// * `pollout`: If true polls will check if the socket can send a message without blocking.
    ///
    /// # See Also
    /// * [nn_poll(3)](http://nanomsg.org/v1.1.2/nn_poll.html)
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
    ///
    /// # See Also
    ///
    /// * [nn_poll(3)](http://nanomsg.org/v1.1.2/nn_poll.html)
    /// * [`make_poll`](#method.make_poll)
    pub fn poll(polls: &mut [Poll], timeout: i32) -> Result<usize> {
        let nready = unsafe { nn_poll(polls.as_mut_ptr() as *mut nn_pollfd, polls.len() as c_int, timeout) };
        error_guard!(nready);
        Ok(nready as usize)
    }

    /// Get the underlying raw file descriptor for the socket.
    #[inline]
    pub unsafe fn as_raw_fd(&self) -> RawFd {
        self.0
    }

    /// Convert into the underlying raw file descriptor for the socket.
    ///
    /// This consumes the socket. It is the callers responsibility to make sure
    /// that the socket is closed (for example with
    /// [nn_close](http://nanomsg.org/v1.1.2/nn_close.html)).
    #[inline]
    pub unsafe fn into_raw_fd(self) -> RawFd {
        let fd = self.0;
        mem::forget(self);
        fd
    }

    /// Create a `Socket` from a raw file descriptor.
    #[inline]
    pub unsafe fn from_raw(fd: RawFd) -> Socket {
        Socket(fd)
    }

    /// Set an option on the socket
    ///
    /// # Arguments
    /// * `level`: the level of the option. This could be `NN_SOL_SOCKET` for general options, the
    /// protocol flag for protocol options, or the transport flag for transport options.
    /// * `option`: the flag for the option to set.
    /// * `value`: the value to set the option to.
    ///
    /// # See Also
    /// * [nn_setsockopt(3)](http://nanomsg.org/v1.1.2/nn_setsockopt.html)
    pub unsafe fn set_option<T: OptionSet>(&self, level: c_int, option: c_int, value: T) -> Result<()> {
        T::set(self.0, level, option, value)
    }

    /// Get an option on the socket
    ///
    /// # Arguments
    /// * `level`: The level of the option. This could be `NN_SOL_SOCKET` for general options, the
    /// protocol flag for protocol options, or the transport flag for transport options.
    /// * `opton`: the flag for the option to get.
    ///
    /// # Returns
    /// The value of the option.
    ///
    /// # See Also
    /// * [nn_getsockopt(3)](http://nanomsg.org/v1.1.2/nn_getsockopt.html)
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

/// Trait for values which can be retrieved from socket options.
pub trait OptionGet: Sized {
    /// Get an option for a socket.
    ///
    /// This should call
    /// [nn_getsockopt](https://docs.rs/nanomsg-sys/0.6.2/nanomsg_sys/fn.nn_getsockopt.html)
    /// with the arguments it is passed and deserialize the result.
    ///
    /// # Arguments
    /// * `fd`: the handle for the socket.
    /// * `level`: the level of the option.
    /// * `option`: the option to get.
    ///
    /// # See also
    /// * [nn_getsockopt(3)](http://nanomsg.org/v1.1.2/nn_getsockopt.html)
    fn get(fd: c_int, level: c_int, option: c_int) -> Result<Self>;
}

/// Trait for values which can be stored as socket options.
pub trait OptionSet: Sized {
    /// Set an option for a socket.
    ///
    /// This should call
    /// [nn_setsockopt](https://docs.rs/nanomsg-sys/0.6.2/nanomsg_sys/fn.nn_setsockopt.html)
    /// with the arguments it is passed.
    ///
    /// # Arguments
    /// * `fd`: the handle for the socket.
    /// * `level`: the level of the option.
    /// * `option`: the option to set.
    /// * `val`: the value to set the option to
    ///
    /// # See also
    /// * [nn_getsockopt(3)](http://nanomsg.org/v1.1.2/nn_getsockopt.html)
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
