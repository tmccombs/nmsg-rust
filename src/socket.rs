use std::ffi::CString;
use std::ptr;

use nng_sys::*;

use error::Result;
use options::{GetOption, SetOption, Milliseconds};
use message::Message;

pub struct Socket(nng_socket);

// TODO: these should probably have a lifetime dependent on the Socket
pub struct Dialer(nng_dialer);
pub struct Listener(nng_listener);

trait Endpoint: Sized {
    fn new(sock: &Socket, url: &str) -> Result<Self>;
    fn start(&mut self) -> Result<()> {
        self.start_with_flags(0)
    }
    fn start_nonblocking(&mut self) -> Result<()> {
        self.start_with_flags(NNG_FLAG_NONBLOCK)
    }
    fn close(self) -> Result<()>;

    fn start_with_flags(&mut self, flags: i32) -> Result<()>;

    unsafe fn get_option<T: GetOption>(&self, name: OptionName) -> Result<T>;
    unsafe fn set_option<T: SetOption>(&mut self, name: OptionName, value: &T) -> Result<()>;
}

impl Socket {
    pub fn listen(&mut self, url: &str) -> Result<Listener> {
        let curl = CString::new(url)?;
        let mut listener: nng_listener = 0;
        unsafe {
            error_guard!(nng_listen(self.0, curl.as_ptr(), &mut listener, 0));
        }
        Ok(Listener(listener))
    }

    pub fn dial(&mut self, url: &str) -> Result<Dialer> {
        let curl = CString::new(url)?;
        let mut dialer: nng_dialer = 0;
        unsafe {
            error_guard!(nng_dial(self.0, curl.as_ptr(), &mut dialer, 0));
        }
        Ok(Dialer(dialer))
    }

    pub unsafe fn from_raw(raw: nng_socket) -> Socket {
        Socket(raw)
    }

    pub unsafe fn as_raw(&self) -> nng_socket {
        self.0
    }

    pub unsafe fn get_option<T: GetOption>(&self, name: OptionName) -> Result<T> {
        impl_get_option!(nng_getopt, self.0, name)
    }

    pub unsafe fn set_option<T: SetOption>(&mut self, name: OptionName, value: &T) -> Result<()> {
        impl_set_option!(nng_setopt, self.0, name, value)
    }

    pub fn send(&self, msg: Message) -> Result<()> {
        unsafe {
            error_guard!(nng_sendmsg(self.0, msg.into_raw(), 0));
        }
        Ok(())
    }

    pub fn receive(&self) -> Result<Message> {
        let mut msg: *mut nng_msg = ptr::null_mut();
        unsafe {
            error_guard!(nng_recvmsg(self.0, &mut msg, 0));
            Ok(Message::from_raw(msg))
        }
    }

    pub fn device(self, other: Socket) -> Result<()> {
        unsafe {
            error_guard!(nng_device(self.0, other.0));
        }
        Ok(())
    }

    pub fn loopback_device(self) -> Result<()> {
        unsafe {
            error_guard!(nng_device(self.0, 0));
        }
        Ok(())
    }

    // Get Options

    pub fn receive_timeout(&self) -> Milliseconds {
        unsafe { self.get_option(NNG_OPT_RECVTIMEO) }.unwrap()
    }

    pub fn send_timeout(&self) -> Milliseconds {
        unsafe { self.get_option(NNG_OPT_SENDTIMEO) }.unwrap()
    }

    // TODO: file descriptor options

    pub fn send_buffer(&self) -> usize {
        unsafe { self.get_option(NNG_OPT_SENDBUF) }.unwrap()
    }

    pub fn receive_buffer(&self) -> usize {
        unsafe { self.get_option(NNG_OPT_RECVBUF) }.unwrap()
    }


    // TODO: reconnection options

    pub fn name(&self) -> String {
        unsafe { self.get_option(NNG_OPT_SOCKNAME) }.unwrap()
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            nng_close(self.0);
        }
    }
}

impl Endpoint for Listener {
    fn new(sock: &Socket, url: &str) -> Result<Listener> {
        let curl = CString::new(url)?;
        let mut listener: nng_listener = 0;
        unsafe {
            error_guard!(nng_listener_create(&mut listener, sock.as_raw(), curl.as_ptr()));
        }
        Ok(Listener(listener))
    }

    fn close(self) -> Result<()> {
        unsafe {
            error_guard!(nng_listener_close(self.0));
        }
        Ok(())
    }

    fn start_with_flags(&mut self, flags: i32) -> Result<()> {
        unsafe {
            error_guard!(nng_listener_start(self.0, flags));
        }
        Ok(())
    }

    unsafe fn get_option<T: GetOption>(&self, name: OptionName) -> Result<T> {
        impl_get_option!(nng_listener_getopt, self.0, name)
    }

    unsafe fn set_option<T: SetOption>(&mut self, name: OptionName, value: &T) -> Result<()> {
        impl_set_option!(nng_listener_setopt, self.0, name, value)
    }
}

impl Endpoint for Dialer {
    fn new(sock: &Socket, url: &str) -> Result<Dialer> {
        let curl = CString::new(url)?;
        let mut dialer: nng_dialer = 0;
        unsafe {
            error_guard!(nng_dialer_create(&mut dialer, sock.as_raw(), curl.as_ptr()));
        }
        Ok(Dialer(dialer))
    }

    fn close(self) -> Result<()> {
        unsafe {
            error_guard!(nng_dialer_close(self.0));
        }
        Ok(())
    }

    fn start_with_flags(&mut self, flags: i32) -> Result<()> {
        unsafe {
            error_guard!(nng_dialer_start(self.0, flags));
        }
        Ok(())
    }

    unsafe fn get_option<T: GetOption>(&self, name: OptionName) -> Result<T> {
        impl_get_option!(nng_dialer_getopt, self.0, name)
    }

    unsafe fn set_option<T: SetOption>(&mut self, name: OptionName, value: &T) -> Result<()> {
        impl_set_option!(nng_dialer_setopt, self.0, name, value)
    }
}
