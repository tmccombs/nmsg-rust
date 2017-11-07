use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;

use nanomsg_sys::{nn_allocmsg, nn_freemsg, nn_reallocmsg};
use libc::c_void;

#[derive(Debug)]
pub struct MessageBuffer {
    ptr: *mut c_void,
    size: usize
}

/// A buffer of data for zero-copy messages with nanomsg
///
/// This is a buffer of bytes that avoids being copied when sent with
/// nanomsg. Using a `MessageBuffer` can improve performance.
///
/// The `MessageBuffer` implements `Drop` so that it will automatically
/// free its memory when it goes out of scope.
impl MessageBuffer {
    pub fn new(size: usize) -> MessageBuffer {
        let ptr = unsafe { nn_allocmsg(size, 0) };
        if ptr.is_null() {
            panic!("Out of Memory!");
        }
        MessageBuffer {
            ptr,
            size
        }
    }

    /// Resize the buffer.
    ///
    /// This may copy the contents of the buffer.
    pub fn resize(&mut self, new_size: usize) {
        let ptr = unsafe { nn_reallocmsg(self.ptr, new_size) };
        if ptr.is_null() {
            panic!("Out of Memory!");
        }
        self.ptr = ptr;
        self.size = new_size;
    }

    /// Convert the buffer to a raw pointer.
    ///
    /// It is the user's responsibility to free the buffer
    /// with a call to `nn_freemsg` or equivalent.
    pub unsafe fn into_raw(self) -> *mut c_void {
        let ptr = self.ptr;
        mem::forget(self);
        ptr
    }

    /// Create a buffer from a raw pointer.
    ///
    /// The pointer should have been allocated with
    /// `nn_allocmsg` or equivalent, with a size of `size`.
    pub unsafe fn from_raw(ptr: *mut c_void, size: usize) -> MessageBuffer {
        MessageBuffer {
            ptr,
            size
        }
    }
}

impl Drop for MessageBuffer {
    fn drop(&mut self) {
        unsafe {
            nn_freemsg(self.ptr);
        }
    }
}

impl Deref for MessageBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.ptr as *const u8, self.size)
        }
    }
}

impl DerefMut for MessageBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.ptr as *mut u8, self.size)
        }
    }
}

