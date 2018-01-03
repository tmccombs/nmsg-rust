use std::cmp::min;
use std::io;
use std::mem;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice;

use nng_sys::*;

use pipe::Pipe;

/// A nanomsg message.
///
/// This message contains data that can be sent or received with
/// zero-copy semantics over a nanomsg socket.
pub struct Message {
    ptr: *mut nng_msg
}

/// The header of a nanomsg message.
pub struct MessageHeader<'a> {
    ptr: *mut nng_msg,
    phantom: PhantomData<&'a mut Message>
}

macro_rules! oom_check {
    ($res:expr) => ( assert!($res == 0, "Out of Memory!") )
}

impl Message {
    /// Create a new `Message` with a specified capacity.
    ///
    /// This will be able to hold `cap` bytes of data without reallocating.
    pub fn with_capacity(cap: usize) -> Message {
        unsafe {
            let mut ptr: *mut nng_msg = mem::uninitialized();
            oom_check!(nng_msg_alloc(&mut ptr, cap));
            Message {
                ptr: ptr
            }
        }
    }

    /// Get the length of the message.
    ///
    /// This is the number of bytes actually used, which may
    /// be less than the overall capacity.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { nng_msg_len(self.ptr) }
    }

    /// Reserve capacity for at least `additional` more bytes to be appended.
    ///
    /// After the call capacity is garanteed to be at least `self.len() + additional`.
    pub fn reserve(&mut self, additional: usize) {
        unsafe {
            oom_check!(nng_msg_realloc(self.ptr, self.len() + additional));
        }
    }

    /// Shortens the message, keeping the first `len` bytes and dropping the rest.
    pub fn truncate(&mut self, len: usize) {
        let old_len = self.len();
        if len < old_len {
            unsafe { nng_msg_chop(self.ptr, old_len - len) };
        }
    }

    /// Extracts a slice containing the entire message.
    ///
    /// Equivalent to &s[..].
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self
    }

    /// Extracts a mutable slice containing the entire message.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut *self
    }

    /// Get the header data as a slice.
    pub fn header(&self) -> &[u8] {
        unsafe { header_slice(self.ptr) }
    }

    /// Get a mutable references to the header data.
    #[inline]
    pub fn header_mut(&mut self) -> MessageHeader {
        MessageHeader {
            ptr: self.ptr,
            phantom: PhantomData
        }
    }

    /// Append bytes to the end of the message.
    pub fn append(&mut self, data: &[u8]) {
        unsafe {
            oom_check!(nng_msg_append(self.ptr, data.as_ptr() as *const _, data.len()));
        }
    }

    /// Prepend bytes to the beginning of the message.
    pub fn prepend(&mut self, data: &[u8]) {
        unsafe {
            oom_check!(nng_msg_insert(self.ptr, data.as_ptr() as *const _, data.len()));
        }
    }

    // TODO: u32 functions

    /// Get the pipe that was used to send the message
    pub fn pipe(&self) -> Pipe {
        unsafe {
            Pipe::from_raw(nng_msg_get_pipe(self.ptr))
        }
    }

    /// Set the pipe the message will be sent over
    pub fn set_pipe(&mut self, pipe: &Pipe) {
        unsafe {
            nng_msg_set_pipe(self.ptr, pipe.as_raw());
        }
    }

    /// Clear all message data.
    ///
    /// After this has been called the message length should be zero.
    pub fn clear(&mut self) {
        unsafe { nng_msg_clear(self.ptr) }
    }

    /// Create a Message from a raw `nng_msg` pointer.
    pub unsafe fn from_raw(ptr: *mut nng_msg) -> Message {
        Message {
            ptr: ptr
        }
    }

    /// Get the internal `nng_msg` pointer.
    ///
    /// The `Message` object retains ownership of the pointer.
    pub unsafe fn as_ptr(&self) -> *const nng_msg {
        self.ptr
    }

    /// Returns an unsafe mutable pointer to the internal `nng_msg`.
    ///
    /// The `Message` object retains ownership of the pointer.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut nng_msg {
        self.ptr
    }

    /// Convert into an unsafe mutable pointer to the internal `nng_msg`.
    ///
    /// Ownership of the pointer is transfered to the caller, and the caller is
    /// responsible for freeing the message.
    pub unsafe fn into_raw(self) -> *mut nng_msg {
        let ptr = self.ptr;
        mem::forget(self);
        ptr
    }
}

impl io::Write for Message {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.append(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for Message {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = min(buf.len(), self.len());
        unsafe {
            buf.copy_from_slice(slice::from_raw_parts(nng_msg_body(self.ptr) as *const u8, n));
            // trim off the bytes that we have read.
            nng_msg_trim(self.ptr, n);
        }
        Ok(n)
    }

    // specialize read_to_end to minimize allocations
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        buf.extend_from_slice(&self);
        self.clear();
        Ok(self.len())
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe { nng_msg_free(self.ptr) };
    }
}

impl Clone for Message {
    fn clone(&self) -> Message {
        unsafe {
            let mut ptr: *mut nng_msg = mem::uninitialized();
            oom_check!(nng_msg_dup(&mut ptr, self.ptr));
            Message {
                ptr: ptr
            }
        }
    }
}

impl Deref for Message {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(nng_msg_body(self.ptr) as *const u8, nng_msg_len(self.ptr))
        }
    }
}

impl DerefMut for Message {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(nng_msg_body(self.ptr) as *mut u8, nng_msg_len(self.ptr))
        }
    }
}


impl<'a> MessageHeader<'a> {
    /// Append bytes to the header.
    pub fn append(&mut self, data: &[u8]) {
        unsafe {
            oom_check!(nng_msg_header_append(self.ptr, data.as_ptr() as *const _, data.len()))
        };
    }

    /// Prepend bytes to the header.
    pub fn prepend(&mut self, data: &[u8]) {
        unsafe {
            oom_check!(nng_msg_header_insert(self.ptr, data.as_ptr() as *const _, data.len()))
        };
    }

    /// Get the length of the header.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { nng_msg_header_len(self.ptr) }
    }

    /// Shortens the message header, keeping the first `len` bytes and dropping the rest.
    pub fn truncate(&mut self, len: usize) {
        let old_len = self.len();
        if len < old_len {
            unsafe { nng_msg_header_chop(self.ptr, old_len - len) };
        }
    }
    // TODO u32 methods

}

impl<'a> Deref for MessageHeader<'a> {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &[u8] {
        unsafe { header_slice(self.ptr) }
    }
}

impl<'a> DerefMut for MessageHeader<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(nng_msg_header(self.ptr) as *mut u8, nng_msg_header_len(self.ptr))
        }
    }
}

unsafe fn header_slice<'a>(ptr: *const nng_msg) -> &'a [u8] {
    slice::from_raw_parts(nng_msg_header(ptr as *mut nng_msg) as *const u8, nng_msg_header_len(ptr))
}
