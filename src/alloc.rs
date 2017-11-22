//! Module for allocating zero-copy buffers for nanomsg.
use std::borrow::{Borrow, BorrowMut};
use std::mem;
use std::ops::{Deref, DerefMut, Index, IndexMut, Range, RangeFull, RangeTo, RangeFrom};
use std::slice;

use nanomsg_sys::{nn_allocmsg, nn_freemsg, nn_reallocmsg};
use libc::{c_void, memset};

/// A buffer of data for zero-copy messages with nanomsg
///
/// This is a buffer of bytes that avoids being copied when sent or received with
/// nanomsg. Using a `MessageBuffer` can improve performance.
///
/// The `MessageBuffer` implements `Drop` so that it will automatically
/// free its memory when it goes out of scope.
#[derive(Debug)]
pub struct MessageBuffer {
    ptr: *mut c_void,
    size: usize
}

impl MessageBuffer {
    /// Create a new `MessageBuffer` of the given size.
    ///
    /// # Note
    ///
    /// The contents of the buffer is uninitialized. Use `zeroed` if you want
    /// it to be initially filled with zeros.
    pub fn new(size: usize) -> MessageBuffer {
        let ptr = unsafe { nn_allocmsg(size, 0) };
        assert!(!ptr.is_null(), "Out of Memory!");

        MessageBuffer {
            ptr,
            size
        }
    }

    /// Create a new `MessageBuffer` that is initialized with zeros.
    pub fn zeroed(size: usize) -> MessageBuffer {
        let ptr = unsafe {
            let p = nn_allocmsg(size, 0);
            assert!(!p.is_null(), "Out of Memory!");
            memset(p, 0, size);
            p
        };
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

    /// The length of the `MessageBuffer` in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    /// Extracts a slice containing the entire buffer.
    ///
    /// Equivalent to `&s[...]`.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self
    }

    /// Extracts a mutable slice of the entire buffer.
    ///
    /// Equivalent to `&mut s[...]`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self
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
        assert!(!ptr.is_null());
        MessageBuffer {
            ptr,
            size
        }
    }
}

impl Drop for MessageBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            nn_freemsg(self.ptr);
        }
    }
}

impl Deref for MessageBuffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.ptr as *const u8, self.size)
        }
    }
}

impl DerefMut for MessageBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.ptr as *mut u8, self.size)
        }
    }
}

impl Borrow<[u8]> for MessageBuffer {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl BorrowMut<[u8]> for MessageBuffer {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self
    }
}

impl AsRef<[u8]> for MessageBuffer {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl<'a> From<&'a [u8]> for MessageBuffer {
    fn from(buf: &[u8]) -> MessageBuffer {
        let mut res = MessageBuffer::new(buf.len());
        res.copy_from_slice(buf);
        res
    }
}

impl<'a> From<&'a str> for MessageBuffer {
    fn from(s: &str) -> MessageBuffer {
        MessageBuffer::from(s.as_bytes())
    }
}

impl From<String> for MessageBuffer {
    fn from(s: String) -> MessageBuffer {
        MessageBuffer::from(s.as_bytes())
    }
}

impl From<Vec<u8>> for MessageBuffer {
    fn from(v: Vec<u8>) -> MessageBuffer {
        MessageBuffer::from(v.as_slice())
    }
}

impl Into<Vec<u8>> for MessageBuffer {
    fn into(self) -> Vec<u8> {
        Vec::from(self.as_slice())
    }
}

// Index implementations

macro_rules! def_index_op {
    ($range:ty, $out:ty) => {
        impl Index<$range> for MessageBuffer {
            type Output = $out;

            fn index(&self, index: $range) -> &$out {
                Index::index(&**self, index)
            }
        }

        impl IndexMut<$range> for MessageBuffer {
            fn index_mut(&mut self, index: $range) -> &mut $out {
                IndexMut::index_mut(&mut **self, index)
            }
        }
    };
    ($t:ty) => {
        def_index_op!($t, [u8]);
    }
}

def_index_op!(usize, u8);
def_index_op!(Range<usize>);
def_index_op!(RangeTo<usize>);
def_index_op!(RangeFrom<usize>);

impl Index<RangeFull> for MessageBuffer {
    type Output = [u8];

    fn index(&self, _: RangeFull) -> &[u8] {
        &**self
    }
}

impl IndexMut<RangeFull> for MessageBuffer {
    fn index_mut(&mut self, _: RangeFull) -> &mut [u8] {
        &mut **self
    }
}

