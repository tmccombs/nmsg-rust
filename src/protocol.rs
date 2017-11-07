
use nanomsg_sys::*;

use alloc::MessageBuffer;
use socket::{Socket, Endpoint, Flags, Domain, Protocol};
use error::{Error, Result};

macro_rules! sock_option {
    ($getter:ident, $setter:ident = $opt:ident<$t:ty>) => {
        fn $getter(&self) -> $t {
            unsafe { self.socket().get_option::<$t>(NN_SOL_SOCKET, $opt) }.unwrap()
        }
        fn $setter(&mut self, val: $t) -> Result<()> {
            unsafe { self.socket_mut().set_option::<$t>(NN_SOL_SOCKET, $opt, val) }
        }
    };
}

const NN_MAXTTL: c_int = 17;

pub trait SPSocket {
    type Companion: SPSocket;

    fn socket(&self) -> &Socket;
    fn socket_mut(&mut self) -> &mut Socket;
    fn protocol(&self) -> Protocol;

    #[inline]
    fn bind(&mut self, addr: &str) -> Result<Endpoint> {
        self.socket_mut().bind(addr)
    }

    #[inline]
    fn connect(&mut self, addr: &str) -> Result<Endpoint> {
        self.socket_mut().connect(addr)
    }

    fn loopback_device(&self) -> Error {
        Socket::loopback_device(self.socket())
    }

    fn device(&self, companion: &Self::Companion) -> Error where Self: Sized {
        Socket::device(self.socket(), companion.socket())
    }

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

    sock_option!(reconnect_interval, set_reconnect_interval = NN_RECONNECT_IVL<i32>);
    sock_option!(max_reconnect_interval, set_max_reconnect_interval = NN_RECONNECT_IVL_MAX<i32>);
    sock_option!(max_ttl, set_max_ttl = NN_MAXTTL<i32>);
    sock_option!(ipv4_only, set_ipv4_only = NN_IPV4ONLY<bool>);
}

pub trait SPRecv: SPSocket {
    #[inline]
    fn recv(&self) -> Result<MessageBuffer> {
        self.socket().recv(Flags::empty())
    }

    #[inline]
    fn recv_nb(&self) -> Result<MessageBuffer> {
        self.socket().recv(Flags::DONTWAIT)
    }

    #[inline]
    fn recv_buf(&self, buffer: &mut [u8]) -> Result<usize> {
        self.socket().recv_buf(buffer, Flags::empty())
    }

    #[inline]
    fn recv_buf_nb(&self, buffer: &mut [u8]) -> Result<usize> {
        self.socket().recv_buf(buffer, Flags::DONTWAIT)
    }

    sock_option!(rcv_buffer, set_rcv_buffer = NN_RCVBUF<i32>);
    sock_option!(rcv_max_size, set_rcv_max_size = NN_RCVMAXSIZE<i32>);
    sock_option!(rcv_timeout, set_rcv_timeout = NN_RCVTIMEO<i32>);
    sock_option!(rcv_priority, set_rcv_priority = NN_RCVPRIO<i32>);
}

pub trait SPSend: SPSocket {

    #[inline]
    fn send(&self, buffer: &[u8]) -> Result<usize> {
        self.socket().send(buffer, Flags::empty())
    }

    #[inline]
    fn send_nb(&self, buffer: &[u8]) -> Result<usize> {
        self.socket().send(buffer, Flags::DONTWAIT)
    }

    #[inline]
    fn send_msg(&self, buffer: MessageBuffer) -> Result<usize> {
        self.socket().send_msg(buffer, Flags::empty())
    }

    #[inline]
    fn send_msg_nb(&self, buffer: MessageBuffer) -> Result<usize> {
        self.socket().send_msg(buffer, Flags::DONTWAIT)
    }

    sock_option!(send_buffer, set_send_buffer = NN_SNDBUF<i32>);
    sock_option!(send_timeout, set_send_timeout = NN_SNDTIMEO<i32>);
    sock_option!(send_priority, set_send_priority = NN_SNDPRIO<i32>);
}

macro_rules! def_protocol {
    ($name:ident : $($extra:ident),+ <> $comp:ident) => {
        pub struct $name {
            sock: Socket
        }

        impl $name {
            pub fn new() -> Result<$name> {
                Ok($name {
                    sock: Socket::new(Domain::SP, Protocol::$name)?
                })
            }

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
            fn socket_mut(&mut self) -> &mut Socket {
                &mut self.sock
            }
        }

        $(impl $extra for $name { })+
    };
}

def_protocol!(Pub: SPSend <> Sub);
def_protocol!(Sub: SPRecv <> Pub);
def_protocol!(Bus: SPSend, SPRecv <> Bus);
def_protocol!(Req: SPSend, SPRecv <> Rep);
def_protocol!(Rep: SPSend, SPRecv <> Req);
def_protocol!(Push: SPSend <> Pull);
def_protocol!(Pull: SPRecv <> Push);
def_protocol!(Surveyor: SPSend, SPRecv <> Respondent);
def_protocol!(Respondent: SPSend, SPRecv <> Surveyor);
def_protocol!(Pair: SPSend, SPRecv <> Pair);
