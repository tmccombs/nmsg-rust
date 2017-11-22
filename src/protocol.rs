//! Higher level interface for nanomsg sockets.
//!
//! This contains higher-level, type-safe, rusty interfaces for the
//! Nanomsg Scalability Protocols. These interfaces provide additional safety
//! by only only implementing methods on the types of
//! sockets that support those operations,
use std::result;
#[cfg(unix)]
use std::os::unix::io::RawFd as PollFd;
#[cfg(windows)]
use std::os::windows::io::RawSocket as PollFd;

use nanomsg_sys::*;

use alloc::MessageBuffer;
use socket::{Socket, Endpoint, Flags, Domain, Protocol};
use error::{Error, Result};

macro_rules! sock_option {
    ($getter:ident, $setter:ident = $opt:ident<$t:ty>, $description:expr) => {
        #[doc = "Get "]
        #[doc = $description]
        #[doc = "\n\n#See Also"]
        ///
        /// * [nn_getsockopt(3)](http://nanomsg.org/v1.1.2/nn_getsockopt.html)
        fn $getter(&self) -> $t {
            unsafe { self.socket().get_option::<$t>(NN_SOL_SOCKET, $opt) }.unwrap()
        }
        #[doc = "Set "]
        #[doc = $description]
        #[doc = "\n\n#See Also"]
        ///
        /// * [nn_setsockopt(3)](http://nanomsg.org/v1.1.2/nn_setsockopt.html)
        fn $setter(&self, val: $t) -> Result<()> {
            unsafe { self.socket().set_option::<$t>(NN_SOL_SOCKET, $opt, val) }
        }
    };
}

const NN_MAXTTL: c_int = 17;

/// Scalability Protocol Socket
///
/// This trait contains common functionality for all the Scalability Protocols.
/// It is a wrapper around a raw nanomsg socket, with a more rusty API.
///
/// # See Also
///
/// * [`SPRecv`](trait.SPRecv.html)
/// * [`SPSend`](trait.SPSend.html)
/// * [`Socket`](../socket/struct.Socket.html)
pub trait SPSocket {
    /// The type of the socket on the other connection.
    ///
    /// This is the type that must be used when connecting to another socket
    /// with [`device`](#method.device).
    ///
    /// For example the companion type of `Pub` is `Sub` and the companion type of
    /// `Pair` is `Pair`.
    type Companion: SPSocket;

    /// Get a reference to the internal socket.
    ///
    /// In most cases this shouldn't be used by user code.
    fn socket(&self) -> &Socket;

    /// Get the underlying protocol.
    ///
    /// Return a [`Protocol`](../socket/enum.Protocol.html) for the socket.
    /// There is a one-to-one mapping between the provided implementations and the protocol
    /// enumeration values.
    fn protocol(&self) -> Protocol;

    /// Bind the socket to an address.
    ///
    /// Adds a local endpoint to the socket s. The endpoint can be then used by other applications to connect to.
    ///
    /// Note that `bind` and `connect` may be called multiple times on the same socket thus
    /// allowing the socket to communicate with multiple heterogeneous endpoints.
    ///
    /// #Arguments
    ///
    /// The `addr` argument consists of two parts as follows: transport://address. The transport specifies the underlying transport protocol to use. The meaning of the address part
    /// is specific to the underlying transport protocol.
    ///
    /// For the list of available transport protocols check the list on [nanomsg(7) manual page](http://nanomsg.org/v1.1.2/nanomsg.html).
    ///
    /// Maximum length of the `addr` parameter is specified by [nanomsg_sys::NN_SOCKADDR_MAX](../../nanomsg_sys/constant.NN_SOCKADDR_MAX).
    ///
    /// # Returns
    ///
    /// If successful, an [`Endpoint`](../socket/struct.Endpoint.html) will be returned, which
    /// can be used to close the endpoint. Otherwise, an error is returned.
    ///
    /// # See Also
    ///
    /// * [nn_bind(3)](http://nanomsg.org/v1.1.2/nn_bind.html)
    /// * [`connect`](#method.connect)
    #[inline]
    fn bind(&self, addr: &str) -> Result<Endpoint> {
        self.socket().bind(addr)
    }

    /// Connect the socket to an address.
    ///
    /// Adds a remote endpoint to the socket s. The library would then try to connect to the
    /// specified remote endpoint.
    ///
    /// Note that `connect` and `bind` may be called multiple times on the same socket thus
    /// allowing the socket to communicate with multiple heterogeneous endpoints.
    ///
    /// # Arguments
    ///
    /// The `addr` argument consists of two parts as follows: transport://address. The transport
    /// specifies the underlying transport protocol to use. The meaning of the address part is
    /// specific to the underlying transport protocol.
    ///
    /// For the list of available transport protocols check the list on [nanomsg(7) manual page](http://nanomsg.org/v1.1.2/nanomsg.html).
    ///
    /// Maximum length of the `addr` parameter is specified by [nanomsg_sys::NN_SOCKADDR_MAX](../../nanomsg_sys/constant.NN_SOCKADDR_MAX.html).
    ///
    /// # Returns
    ///
    /// If successful, an [`Endpoint`](../socket/struct.Endpoint.html) will be returned, which
    /// can be used to close the endpoint. Otherwise, an error is returned.
    ///
    /// # See Also
    ///
    /// * [nn_connect(3)](http://nanomsg.org/v1.1.2/nn_connect.html)
    /// * [`bind`](#method.bind)
    #[inline]
    fn connect(&self, addr: &str) -> Result<Endpoint> {
        self.socket().connect(addr)
    }

    /// Starts a device to forward messages between two sockets.
    ///
    /// This method loops and sends any messages received from `self` to `companion`.
    ///
    /// # Returns
    ///
    /// This will only return if nanomsg encounters an error. In that case the error encountered
    /// is returned. One way it could return is if `Socket::terminate` is called, closing all open
    /// sockets.
    ///
    /// # See Also
    ///
    /// * [nn_device(3)](http://nanomsg.org/v1.1.2/nn_device.html)
    /// * [`Socket::terminate`](../socket/struct.Socket.html#method.terminate)
    fn device(&self, companion: &Self::Companion) -> Error where Self: Sized {
        Socket::device(self.socket(), companion.socket())
    }

    /// Get the sockets domain.
    ///
    /// # See Also
    ///
    /// * [nn_getsockopt(3)](http://nanomsg.org/v1.1.2/nn_getsockopt.html)
    fn domain(&self) -> Domain {
        let dom = unsafe {
            self.socket().get_option::<i32>(NN_SOL_SOCKET, NN_DOMAIN)
        };
        match dom.unwrap() {
            AF_SP => Domain::SP,
            AF_SP_RAW => Domain::SPRaw,
            _ => panic!("Unknown domain")

        }
    }

    sock_option!(reconnect_interval, set_reconnect_interval = NN_RECONNECT_IVL<i32>,
                 "how long to wait to re-establish a broken connection in milliseconds.");
    sock_option!(max_reconnect_interval, set_max_reconnect_interval = NN_RECONNECT_IVL_MAX<i32>,
                 "the maximum interval in milliseconds between re-establish attempts during exponential backoff.");
    sock_option!(max_ttl, set_max_ttl = NN_MAXTTL<i32>, "maximum number of hops a message can go before being dropped");
    sock_option!(ipv4_only, set_ipv4_only = NN_IPV4ONLY<bool>, "whether IPv6 addresses are supported.");

    /// Return true if Nagle's algorithm is disabled.
    ///
    /// This will fail if the underlying transport isn't TCP.
    ///
    /// # See Also
    ///
    /// * [nn_tcp(7)](http://nanomsg.org/v1.1.2/nn_tcp.html)
    fn tcp_nodelay(&self) -> Result<bool> {
        unsafe {
            self.socket().get_option::<bool>(NN_TCP, NN_TCP_NODELAY)
        }
    }

    /// Disable (or enable) Nagle's algorithm.
    ///
    /// This will fail if the underlying transport isn't TCP.
    ///
    /// # See Also
    ///
    /// * [nn_tcp(7)](http://nanomsg.org/v1.1.2/nn_tcp.html)
    fn set_tcp_nodelay(&self, delay: bool) -> Result<()> {
        unsafe {
            self.socket().set_option(NN_TCP, NN_TCP_NODELAY, delay)
        }
    }
}

/// Trait for sockets which are able to receive messages.
pub trait SPRecv: SPSocket {
    /// Receive a message.
    ///
    /// Blocks until a message can be read.
    #[inline]
    fn recv(&self) -> Result<MessageBuffer> {
        self.socket().recv(Flags::empty())
    }

    /// Receive a message without blocking.
    ///
    /// If receiving would block `Err(error::WOULD_BLOCK)` is returned.
    #[inline]
    fn recv_nb(&self) -> Result<MessageBuffer> {
        self.socket().recv(Flags::DONTWAIT)
    }

    /// Receive a message into a buffer.
    ///
    /// Blocks until a message can be read.
    ///
    /// # Arguments
    ///
    /// The message is written into `buffer` if successful.
    /// If the message length exceeds `buffer.len()` the message
    /// will be truncated.
    ///
    /// # Returns
    ///
    /// The number of bytes in the message if successful. Note
    /// that this may be greater than the size of the buffer.
    #[inline]
    fn recv_buf(&self, buffer: &mut [u8]) -> Result<usize> {
        self.socket().recv_buf(buffer, Flags::empty())
    }

    /// Receive a message into a buffer without blocking.
    ///
    /// # Arguments
    ///
    /// The message is written into `buffer` if successful.
    /// If the message length exceeds `buffer.len()` the message
    /// will be truncated.
    ///
    /// # Retursn
    ///
    /// The number of bytes in the message if successful. Note
    /// that this may be greater than the size of the buffer.
    ///
    /// If receiving would block `Err(error::WOULD_BLOCK)` is returned.
    #[inline]
    fn recv_buf_nb(&self, buffer: &mut [u8]) -> Result<usize> {
        self.socket().recv_buf(buffer, Flags::DONTWAIT)
    }

    sock_option!(rcv_buffer, set_rcv_buffer = NN_RCVBUF<i32>, "size of the receive buffer");
    sock_option!(rcv_max_size, set_rcv_max_size = NN_RCVMAXSIZE<i32>,
                 "the maximum message size.\n\nA negative value means it is limited only be available memory.");
    sock_option!(rcv_timeout, set_rcv_timeout = NN_RCVTIMEO<i32>, "the timeout for receive operations in milliseconds.");
    sock_option!(rcv_priority, set_rcv_priority = NN_RCVPRIO<i32>, "the receiving priority for subsequently added endpoints.");

    /// Get a raw file descriptor that is readable when a message can be received.
    ///
    /// This file descriptor should only be used to poll if the socket is
    /// available for reading.
    ///
    /// # See Also
    ///
    /// * [nn_getsockopt(3)](http://nanomsg.org/v1.1.2/nn_getsockopt.html)
    fn recv_poll_fd(&self) -> Result<PollFd> {
        unsafe {
            self.socket().get_option::<PollFd>(NN_SOL_SOCKET, NN_RCVFD)
        }
    }
}

/// Trait for sockets which are able to send messages
pub trait SPSend: SPSocket {
    /// Send a message.
    ///
    /// Blocks until the message can be sent.
    ///
    /// # Returns
    ///
    /// The number of bytes in the message.
    #[inline]
    fn send(&self, buffer: MessageBuffer) -> Result<usize> {
        self.socket().send(buffer, Flags::empty())
    }

    /// Send a message without blocking.
    ///
    /// If sending would block `Err(error::WOULD_BLOCK)` is returned.
    ///
    /// # Returns
    ///
    /// The number of bytes in the message.
    #[inline]
    fn send_nb(&self, buffer: MessageBuffer) -> Result<usize> {
        self.socket().send(buffer, Flags::DONTWAIT)
    }

    /// Send a message from a slice.
    ///
    /// Unlike `send` this will copy the data from the slice
    /// before sending it. Blocks until the message can be sent.
    ///
    /// # Returns
    ///
    /// The number of bytes in the message.
    #[inline]
    fn send_buf(&self, buffer: &[u8]) -> Result<usize> {
        self.socket().send_buf(buffer, Flags::empty())
    }

    /// Send a message from a slice without blocking.
    ///
    /// Unlike `send_nb` this will copy the data from the sice
    /// before sending it. If sending would block
    /// `Err(error::WOULD_BLOCK)` is returned.
    ///
    /// # Returns
    ///
    /// The number of bytes in the message.
    #[inline]
    fn send_buf_nb(&self, buffer: &[u8]) -> Result<usize> {
        self.socket().send_buf(buffer, Flags::DONTWAIT)
    }

    sock_option!(send_buffer, set_send_buffer = NN_SNDBUF<i32>, "size of the send buffer");
    sock_option!(send_timeout, set_send_timeout = NN_SNDTIMEO<i32>, "the timeout for send operations in milliseconds.");
    sock_option!(send_priority, set_send_priority = NN_SNDPRIO<i32>, "the sending priority for subsequently added endpoints.");

    /// Get a raw file descriptor that is readable when a message can be sent.
    ///
    /// This file descriptor should only be used to poll if the socket is
    /// available for writing.
    ///
    /// # See Also
    ///
    /// * [nn_getsockopt(3)](http://nanomsg.org/v1.1.2/nn_getsockopt.html)
    fn send_poll_fd(&self) -> Result<PollFd> {
        unsafe {
            self.socket().get_option::<PollFd>(NN_SOL_SOCKET, NN_SNDFD)
        }
    }
}

/// Trait for sockets which can be looped back to themselves.
///
/// This only applies to sockets which can communicate with sockets of the same time, such as Pair
/// and Bus.
pub trait Loopback: SPSocket {
    /// Create a loopback device that forwards all traffic back to this socket.
    ///
    /// This function will run in an infinite loop, until an error is encountered, or
    /// [`Socket::terminate`](../socket/struct.Socket.html#method.terminate) is called.
    ///
    /// Roughly equivalent to `s.device(s)`.
    ///
    /// # See Also
    ///
    /// * [nn_device(3)](http://nanomsg.org/v1.1.2/nn_device.html)
    /// * [`SPSocket::device`](../trait.SPSocket.html#method.device)
    #[inline]
    fn loopback_device(&self) -> Error {
        Socket::loopback_device(self.socket())
    }
}

macro_rules! def_protocols {
    ($($(#[$attrs:meta])* struct $name:ident : $($extra:ident),+ <> $comp:ident ;)*) => {$(
            $(#[$attrs])*
            pub struct $name {
                sock: Socket
            }

            impl $name {
                /// Create a new socket
                pub fn new() -> Result<$name> {
                    Ok($name {
                        sock: Socket::new(Domain::SP, Protocol::$name)?
                    })
                }

                /// Create a new raw socket that is suitable for creating devices.
                ///
                /// See [`SPSocket::device`](../trait.SPSocket.html#method.device)
                pub fn new_raw() -> Result<$name> {
                    Ok($name {
                        sock: Socket::new(Domain::SPRaw, Protocol::$name)?
                    })
                }
            }

            impl SPSocket for $name {
                type Companion = $comp;

                fn protocol(&self) -> Protocol {
                    Protocol::$name
                }

                fn socket(&self) -> &Socket {
                    &self.sock
                }
            }

            $(impl $extra for $name { })+
    )*};
}

def_protocols!{
    /// Publish socket
    ///
    /// A socket which can be used to publish messages to any subscribe sockets
    /// which have subscribed to those messages.
    ///
    /// # See Also
    /// * [nn_pubsub(7)](http://nanomsg.org/v1.1.2/nn_pubsub.html)
    /// * [`Sub`](../struct.Sub.html)
    struct Pub: SPSend <> Sub;
    /// Subscribe socket
    ///
    /// A socket which can subscribe to messages from one or more Publish sockets.
    ///
    /// # See Also
    /// * [nn_pubsub(7)](http://nanomsg.org/v1.1.2/nn_pubsub.html)
    /// * [`Pub`](../struct.Pub.html)
    struct Sub: SPRecv <> Pub;
    /// Bus socket
    ///
    /// A socket that can broadcast messages to all other sockets on the bus, and
    /// receive messages from all other sockets.
    ///
    /// # See Also
    /// * [nn_bus(7)](http://nanomsg.org/v1.1.2/nn_bus.html)
    struct Bus: SPSend, SPRecv, Loopback <> Bus;
    /// Request socket
    ///
    /// A socket which can make requests (like RPC) to a Reply socket, and
    /// receive the reply.
    ///
    /// # See Also
    /// * [nn_reqrep(7)](http://nanomsg.org/v1.1.2/nn_reqrep.html)
    /// * [`Rep`](../struct.Rep.html)
    struct Req: SPSend, SPRecv <> Rep;
    /// Reply socket
    ///
    /// A socket which replies to requests (like RPC) from a Request socket.
    ///
    /// # See Also
    /// * [nn_reqrep(7)](http://nanomsg.org/v1.1.2/nn_reqrep.html)
    /// * [`Req`](../struct.Req.html)
    struct Rep: SPSend, SPRecv <> Req;
    /// Push socket
    ///
    /// A socket which can push messages to a pipeline/queue.
    ///
    /// # See Also
    /// * [nn_pipeline(7)](http://nanomsg.org/v1.1.2/nn_pipeline.html)
    /// * [`Pull`](../struct.Pull.html)
    struct Push: SPSend <> Pull;
    /// Pull socket
    ///
    /// A socket which can pull messages from a pipeline/queue.
    ///
    /// # See Also
    /// * [nn_pipeline(7)](http://nanomsg.org/v1.1.2/nn_pipeline.html)
    /// * [`Push`](../struct.Push.html)
    struct Pull: SPRecv <> Push;
    /// Surveyor socket
    ///
    /// A socket which can send out surveys to Respondent sockets.
    ///
    /// # See Also
    /// * [nn_survey(7)](http://nanomsg.org/v1.1.2/nn_survey.html)
    /// * [`Respondent`](../struct.Respondent.html)
    struct Surveyor: SPSend, SPRecv <> Respondent;
    /// Respondent socket
    ///
    /// A socket which can respond to surveys from a Surveyor socket.
    ///
    /// # See Also
    /// * [nn_survey(7)](http://nanomsg.org/v1.1.2/nn_survey.html)
    /// * [`Surveyor`](../struct.Surveyor.html)
    struct Respondent: SPSend, SPRecv <> Surveyor;
    /// Pair socket
    ///
    /// A socket which can send and receive messages from another pair socket.
    ///
    /// # See Also
    /// * [nn_pair(7)](http://nanomsg.org/v1.1.2/nn_pair.html)
    struct Pair: SPSend, SPRecv, Loopback <> Pair;
}

impl Sub {
    /// Subscribe to a topic.
    ///
    /// The subscriber socket will only receive messages which begin with
    /// prefixes (topics) that are subscribed to.
    pub fn subscribe(&self, topic: &[u8]) {
        unsafe {
            self.socket().set_option(NN_SUB, NN_SUB_SUBSCRIBE, topic).unwrap();
        }
    }

    /// Unsubscribe from a previously subscribed topic.
    pub fn unsubscribe(&self, topic: &[u8]) {
        unsafe {
            self.socket().set_option(NN_SUB, NN_SUB_UNSUBSCRIBE, topic).unwrap();
        }
    }
}

impl Req {
    /// Send a request and block until we receive a reply
    pub fn request(&self, body: MessageBuffer) -> Result<MessageBuffer> {
        self.send(body)?;
        self.recv()
    }
}

impl Rep {
    /// Reply to a request
    ///
    /// This will block waiting for a request, and once it receives
    /// one, will use the supplied function to prepare a response.
    pub fn reply<F, E>(&self, handler: F) -> result::Result<(), E>
        where F: Fn(MessageBuffer) -> result::Result<MessageBuffer, E>,
              E: From<Error>
    {
        let request = self.recv()?;
        self.send(handler(request)?)?;
        Ok(())
    }

    /// Wait in a loop, replying to messages with
    /// the supplied handler.
    ///
    /// If an error is encountered the loop will be stopped and the
    /// error returned.
    pub fn reply_loop<F, E>(&self, handler: &F) -> E
        where F: Fn(MessageBuffer) -> result::Result<MessageBuffer, E>,
              E: From<Error>
    {
        loop {
            if let Err(e) = self.reply(handler) {
                return e;
            }
        }
    }
}

impl Surveyor {
    /// Send out a survey and wait for responses.
    ///
    /// This sends out `message` as a survey, and awaits responses. Once the survey deadline
    /// expires a vector of the response received is returned.
    pub fn survey(&self, message: MessageBuffer) -> Result<Vec<MessageBuffer>> {
        use error::TIMED_OUT;
        self.send(message)?;
        let mut responses: Vec<MessageBuffer> = Vec::new();
        loop {
            match self.recv() {
                Ok(resp) => responses.push(resp),
                Err(TIMED_OUT) => {
                    responses.shrink_to_fit();
                    return Ok(responses);
                },
                Err(e) => return Err(e)
            }
        }
    }

    /// Set the survey deadline
    ///
    /// Set how to wait for responses after sending a survey message.
    /// Any responses to a survey after the response will be discarded.
    pub fn set_survey_deadline(&self, deadline: i32) -> Result<()> {
        unsafe { self.socket().set_option(NN_SURVEYOR, NN_SURVEYOR_DEADLINE, deadline) }
    }

    /// Get the survey deadline
    ///
    /// See [`set_survey_deadline`](#method.set_survey_deadline)
    pub fn get_survey_deadline(&self) -> i32 {
        unsafe { self.socket().get_option::<i32>(NN_SURVEYOR, NN_SURVEYOR_DEADLINE).unwrap() }
    }
}

impl Respondent {
    /// Wait for a survey response and then respond to it.
    ///
    /// When the respondent receives a message from the surveyor, handle the response
    /// with the handler, and return.
    pub fn respond<F, E>(&self, mut handler: F) -> result::Result<(), E>
        where F: FnMut(MessageBuffer) -> result::Result<MessageBuffer, E>,
              E: From<Error>
    {
        let request = self.recv()?;
        self.send(handler(request)?)?;
        Ok(())
    }
}
