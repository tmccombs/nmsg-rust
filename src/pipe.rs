use std::ffi::CString;

use libc::c_void;
use nng_sys::*;

use address::SocketAddr;
use error::*;
use options::GetOption;

/// A nanomsg pipe.
///
/// A pipe is a single connection between two nodes.
/// This is primarily used to get information about the other
/// end of a message.
pub struct Pipe(nng_pipe);

impl Pipe {
    /// Close the pipe.
    ///
    /// This can be useful, for example during a connection
    /// notification, to disconnect a pipe that is associated with
    /// an invalid or untrusted remote peer.
    ///
    /// In normal circumstances this doesn't need to be called.
    pub fn close(&self) {
        unsafe {
            nng_pipe_close(self.0);
        }
        // TODO: handle errors?
    }

    /// Get the remote address of the pipe
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        unsafe {
            self.get_option(NNG_OPT_REMADDR).ok()
        }
    }

    pub fn local_addr(&self) -> Option<SocketAddr> {
        unsafe {
            self.get_option(NNG_OPT_LOCADDR).ok()
        }
    }

    /// Create a pipe from a raw `nng_pipe`.
    pub unsafe fn from_raw(raw: nng_pipe) -> Pipe {
        Pipe(raw)
    }

    /// Get the raw `nng_pipe` wrapped by the Pipe
    pub unsafe fn as_raw(&self) -> nng_pipe {
        self.0
    }

    /// Get an option by name.
    ///
    /// * Safety
    /// This method is unsafe, because there is no garantee that the option is of the
    /// specified type, and if the type doesn't match the data returned could be
    /// incorrect, incomplete, or even unsafe.
    ///
    /// If possible use a named method to get the option you want.
    pub unsafe fn get_option<T: GetOption>(&self, name: &str) -> Result<T> {
        let cname = CString::new(name)?;
        T::get_option(|ptr: *mut c_void, size: &mut usize| {
            error_guard!(nng_pipe_getopt(self.0, cname.as_ptr(), ptr, size));
            Ok(())
        })
    }
}

