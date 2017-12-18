use std::mem;
use std::ops::Deref;

use libc::c_void;
use nng_sys::{nng_duration, nng_sockaddr, NNG_AF_UNSPEC};

use address::SocketAddr;
use error::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Milliseconds(nng_duration);

impl Deref for Milliseconds {
    type Target = i32;
    #[inline]
    fn deref(&self) -> &i32 {
        &self.0
    }
}

/// Trait for types that options can be retrieved as.
///
/// Usually, this shouldn't be implemented by user code.
pub trait GetOption: Sized {
    #[doc(hidden)]
    fn get_option<F>(getter: F) -> Result<Self> where F: FnMut(*mut c_void, &mut usize) -> Result<()>;
}

/// Trait for types that options can be set from.
///
/// Usually, this shouldn't be implemented by user code.
pub trait SetOption: Sized {
    #[doc(hidden)]
    fn as_ptr(&self) -> *const c_void {
        self as *const _ as *const c_void
    }
    #[doc(hidden)]
    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}


macro_rules! primitive_option {
    ($t:ty) => {
        impl GetOption for $t {
            fn get_option<F>(mut getter: F) -> Result<$t> where F: FnMut(*mut c_void, &mut usize) -> Result<()> {
                let mut val: $t = 0;
                let mut size = mem::size_of::<$t>();
                getter(&mut val as *mut _ as *mut c_void, &mut size)?;
                if size != mem::size_of::<$t>() {
                    return Err(INVALID);
                }
                Ok(val)
            }
        }

        impl SetOption for $t { }
    }
}

primitive_option!(i32);
primitive_option!(usize);
primitive_option!(u64);


impl GetOption for Vec<u8> {
    fn get_option<F>(mut getter: F) -> Result<Vec<u8>> where F: FnMut(*mut c_void, &mut usize) -> Result<()> {
        const BUF_SIZE: usize = 128;
        let mut val = Vec::with_capacity(BUF_SIZE);
        let mut size = BUF_SIZE;
        getter(val.as_mut_ptr() as *mut c_void, &mut size)?;
        if size > BUF_SIZE {
            // we need more space
            val.reserve_exact(size);
            getter(val.as_mut_ptr() as *mut c_void, &mut size)?;
        }
        assert!(size <= val.capacity(), "Size of option changed between calls");
        unsafe { val.set_len(size) };
        Ok(val)
    }
}

impl GetOption for Milliseconds {
    fn get_option<F>(mut getter: F) -> Result<Milliseconds> where F: FnMut(*mut c_void, &mut usize) -> Result<()> {
        let mut val: nng_duration = 0;
        let mut size = mem::size_of::<nng_duration>();
        getter(&mut val as *mut _ as *mut c_void, &mut size)?;
        if size != mem::size_of::<nng_duration>() {
            return Err(INVALID);
        }
        Ok(Milliseconds(val))
    }
}

impl GetOption for nng_sockaddr {
    fn get_option<F>(mut getter: F) -> Result<nng_sockaddr> where F: FnMut(*mut c_void, &mut usize) -> Result<()> {
        let mut addr: nng_sockaddr = nng_sockaddr { s_family: NNG_AF_UNSPEC };
        let mut size = mem::size_of::<nng_sockaddr>();
        getter(&mut addr as *mut _ as *mut c_void, &mut size)?;
        if size != mem::size_of::<nng_sockaddr>() {
            return Err(INVALID);
        }
        Ok(addr)
    }
}

impl GetOption for SocketAddr {
    fn get_option<F>(getter: F) -> Result<SocketAddr> where F: FnMut(*mut c_void, &mut usize) -> Result<()> {
        nng_sockaddr::get_option(getter).map(SocketAddr::from)
    }
}
