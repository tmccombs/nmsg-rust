#![allow(non_camel_case_types)]
extern crate libc;

use libc::{c_char, c_int, c_void};

pub type AioCallback = extern "C" fn(*mut c_void);

pub type nng_socket = u32;
pub type nng_dialer = u32;
pub type nng_listener = u32;
pub type nng_pipe = u32;
/// A duration in milliseconds
pub type nng_duration = i32;

pub enum nng_msg {}
pub enum nng_snapshot {}
pub enum nng_stat {}
pub enum nng_aio {}

/// Infinite duration
pub const NNG_DURATION_INFINITE: nng_duration = -1;
/// Use the default duration
pub const NNG_DURATION_DEFAULT: nng_duration = -2;
/// Zero duration
pub const NNG_DURATION_ZERO: nng_duration = 0;

pub const NNG_MAXADDRLEN: usize = 128;

pub const NNG_FLAG_ALLOC: c_int = 1;
pub const NNG_FLAG_NONBLOCK: c_int = 2;

// options
pub const NNG_OPT_SOCKNAME: &'static str = "socket-name";
pub const NNG_OPT_RAW: &'static str = "raw";
pub const NNG_OPT_LINGER: &'static str = "linger";
pub const NNG_OPT_RECVBUF: &'static str = "recv-buffer";
pub const NNG_OPT_SENDBUF: &'static str = "send-buffer";
pub const NNG_OPT_RECVFD: &'static str = "recv-fd";
pub const NNG_OPT_SENDFD: &'static str = "send-fd";
pub const NNG_OPT_RECVTIMEO: &'static str = "recv-timeout";
pub const NNG_OPT_SENDTIMEO: &'static str = "send-timeout";
pub const NNG_OPT_LOCADDR: &'static str = "local-address";
pub const NNG_OPT_REMADDR: &'static str = "remote-address";
pub const NNG_OPT_URL: &'static str = "url";
pub const NNG_OPT_MAXTTL: &'static str = "ttl-max";
pub const NNG_OPT_PROTOCOL: &'static str = "protocol";
pub const NNG_OPT_TRANSPORT: &'static str = "transport";
pub const NNG_OPT_RECVMAXSZ: &'static str = "recv-size-max";
pub const NNG_OPT_RECONNMINT: &'static str = "reconnect-time-min";
pub const NNG_OPT_RECONNMAXT: &'static str = "reconnect-time-max";


// Error codes
pub const NNG_EINTR: c_int = 1;
pub const NNG_ENOMEM: c_int = 2;
pub const NNG_EINVAL: c_int = 3;
pub const NNG_EBUSY: c_int = 4;
pub const NNG_ETIMEDOUT: c_int = 5;
pub const NNG_ECONNREFUSED: c_int = 6;
pub const NNG_ECLOSED: c_int = 7;
pub const NNG_EAGAIN: c_int = 8;
pub const NNG_ENOTSUP: c_int = 9;
pub const NNG_EADDRINUSE: c_int = 10;
pub const NNG_ESTATE: c_int = 11;
pub const NNG_ENOENT: c_int = 12;
pub const NNG_EPROTO: c_int = 13;
pub const NNG_EUNREACHABLE: c_int = 14;
pub const NNG_EADDRINVAL: c_int = 15;
pub const NNG_EPERM: c_int = 16;
pub const NNG_EMSGSIZE: c_int = 17;
pub const NNG_ECONNABORTED: c_int = 18;
pub const NNG_ECONNRESET: c_int = 19;
pub const NNG_ECANCELED: c_int = 20;
pub const NNG_ENOFILES: c_int = 21;
pub const NNG_ENOSPC: c_int = 22;
pub const NNG_EEXIST: c_int = 23;
pub const NNG_EREADONLY: c_int = 24;
pub const NNG_EWRITEONLY: c_int = 25;
pub const NNG_EINTERNAL: c_int = 1000;
pub const NNG_ESYSERR: c_int = 0x10000000;
pub const NNG_ETRANERR: c_int = 0x20000000;

// address families
pub const NNG_AF_UNSPEC:u16 = 0;
pub const NNG_AF_INPROC:u16 = 1;
pub const NNG_AF_IPC:u16 = 2;
pub const NNG_AF_INET:u16 = 3;
pub const NNG_AF_INET6:u16 = 4;
pub const NNG_AF_ZT:u16 = 5;

// stats
pub const NNG_STAT_LEVEL: c_int = 0;
pub const NNG_STAT_COUNTER: c_int = 1;

pub const NNG_UNIT_NONE: c_int = 0;
pub const NNG_UNIT_BYTES: c_int = 1;
pub const NNG_UNIT_MESSAGES: c_int = 2;
pub const NNG_UNIT_BOOLEAN: c_int = 3;
pub const NNG_UNIT_MILLIS: c_int = 4;
pub const NNG_UNIT_EVENTS: c_int = 5;


#[repr(C)]
#[derive(Copy, Clone)]
pub struct nng_sockaddr_path {
    pub sa_family: u16,
    pub sa_path: [u8; NNG_MAXADDRLEN]
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct nng_sockaddr_in6 {
    pub sa_family: u16,
    pub sa_port: u16,
    pub sa_addr: [u8; 16]
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct nng_sockaddr_in {
    pub sa_family: u16,
    pub sa_port: u16,
    pub sa_addr: u32
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct nng_sockaddr_zt {
    pub sa_family: u16,
    pub sa_nwid: u64,
    pub sa_nodeid: u64,
    pub sa_port: u32
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union nng_sockaddr {
    pub s_family: u16,
    pub s_path: nng_sockaddr_path,
    pub s_inproc: nng_sockaddr_path,
    pub s_in6: nng_sockaddr_in6,
    pub s_in: nng_sockaddr_in,
    pub s_zt: nng_sockaddr_zt,

}

// limit to static linking for now,
// since nng hasn't stabilized yet
#[link(name = "nng_static", kind = "static")]
extern {
    pub fn nng_fini();

    pub fn nng_close(socket: nng_socket) -> c_int;

    pub fn nng_closeall();

    pub fn nng_setopt(socket: nng_socket, key: *const c_char, value: *const c_void, len: usize) -> c_int;
    pub fn nng_setopt_int(socket: nng_socket, key: *const c_char, value: c_int) -> c_int;
    pub fn nng_setopt_ms(socket: nng_socket, key: *const c_char, value: nng_duration) -> c_int;
    pub fn nng_setopt_size(socket: nng_socket, key: *const c_char, value: usize) -> c_int;
    pub fn nng_setopt_uint64(socket: nng_socket, key: *const c_char, value: u64) -> c_int;

    pub fn nng_getopt(socket: nng_socket, key: *const c_char, value: *mut c_void, len: &mut usize) -> c_int;
    pub fn nng_getopt_int(socket: nng_socket, key: *const c_char, value: &mut c_int) -> c_int;
    pub fn nng_getopt_ms(socket: nng_socket, key: *const c_char, value: &mut nng_duration) -> c_int;
    pub fn nng_getopt_size(socket: nng_socket, key: *const c_char, value: &mut usize) -> c_int;
    pub fn nng_getopt_uint64(socket: nng_socket, key: *const c_char, value: &mut u64) -> c_int;

    pub fn nng_listen(socket: nng_socket, url: *const c_char, listener: &mut nng_listener, flags: c_int) -> c_int;
    pub fn nng_dial(socket: nng_socket, url: *const c_char, dialer: &mut nng_dialer, flags: c_int) -> c_int;

    pub fn nng_dialer_create(result: &mut nng_dialer, socket: nng_socket, url: *const c_char) -> c_int;
    pub fn nng_listener_create(result: &mut nng_listener, socket: nng_socket, url: *const c_char) -> c_int;

    pub fn nng_dialer_start(dialer: nng_dialer, flags: c_int) -> c_int;
    pub fn nng_listener_start(listener: nng_listener, flags: c_int) -> c_int;

    pub fn nng_dialer_close(dialer: nng_dialer) -> c_int;
    pub fn nng_listener_close(listener: nng_listener) -> c_int;


    pub fn nng_dialer_setopt(dialer: nng_dialer, key: *const c_char, value: *const c_void, len: usize) -> c_int;
    pub fn nng_dialer_setopt_int(dialer: nng_dialer, key: *const c_char, value: c_int) -> c_int;
    pub fn nng_dialer_setopt_ms(dialer: nng_dialer, key: *const c_char, value: nng_duration) -> c_int;
    pub fn nng_dialer_setopt_size(dialer: nng_dialer, key: *const c_char, value: usize) -> c_int;
    pub fn nng_dialer_setopt_uint64(dialer: nng_dialer, key: *const c_char, value: u64) -> c_int;

    pub fn nng_dialer_getopt(dialer: nng_dialer, key: *const c_char, value: *mut c_void, len: &mut usize) -> c_int;
    pub fn nng_dialer_getopt_int(dialer: nng_dialer, key: *const c_char, value: &mut c_int) -> c_int;
    pub fn nng_dialer_getopt_ms(dialer: nng_dialer, key: *const c_char, value: &mut nng_duration) -> c_int;
    pub fn nng_dialer_getopt_size(dialer: nng_dialer, key: *const c_char, value: &mut usize) -> c_int;
    pub fn nng_dialer_getopt_uint64(dialer: nng_dialer, key: *const c_char, value: &mut u64) -> c_int;

    pub fn nng_listener_setopt(listener: nng_listener, key: *const c_char, value: *const c_void, len: usize) -> c_int;
    pub fn nng_listener_setopt_int(listener: nng_listener, key: *const c_char, value: c_int) -> c_int;
    pub fn nng_listener_setopt_ms(listener: nng_listener, key: *const c_char, value: nng_duration) -> c_int;
    pub fn nng_listener_setopt_size(listener: nng_listener, key: *const c_char, value: usize) -> c_int;
    pub fn nng_listener_setopt_uint64(listener: nng_listener, key: *const c_char, value: u64) -> c_int;

    pub fn nng_listener_getopt(listener: nng_listener, key: *const c_char, value: *mut c_void, len: &mut usize) -> c_int;
    pub fn nng_listener_getopt_int(listener: nng_listener, key: *const c_char, value: &mut c_int) -> c_int;
    pub fn nng_listener_getopt_ms(listener: nng_listener, key: *const c_char, value: &mut nng_duration) -> c_int;
    pub fn nng_listener_getopt_size(listener: nng_listener, key: *const c_char, value: &mut usize) -> c_int;
    pub fn nng_listener_getopt_uint64(listener: nng_listener, key: *const c_char, value: &mut u64) -> c_int;


    pub fn nng_strerror(errno: c_int) -> *const c_char;

    pub fn nng_send(socket: nng_socket, buf: *mut c_void, len: usize, flags: c_int) -> c_int;
    pub fn nng_recv(socket: nng_socket, buf: *mut c_void, szp: &mut usize, flags: c_int) -> c_int;

    pub fn nng_sendmsg(socket: nng_socket, msg: *mut nng_msg, flags: c_int) -> c_int;
    pub fn nng_recvmsg(socket: nng_socket, msg: &mut *mut nng_msg, flags: c_int) -> c_int;

    pub fn nng_send_aio(socket: nng_socket, ap: *mut nng_aio);
    pub fn nng_recv_aio(socket: nng_socket, ap: *mut nng_aio);

    pub fn nng_alloc(size: usize) -> *mut c_void;
    pub fn nng_free(buf: *mut c_void, size: usize);

    pub fn nng_aio_alloc(buf: &mut *mut nng_aio, cb: Option<AioCallback>, arg: *mut c_void) -> c_int;
    pub fn nng_aio_free(buf: *mut nng_aio);

    pub fn nng_aio_stop(aio: *mut nng_aio);
    pub fn nng_aio_result(aio: *mut nng_aio) -> c_int;
    pub fn nng_aio_cancel(aio: *mut nng_aio);
    pub fn nng_aio_wait(aio: *mut nng_aio);
    pub fn nng_aio_set_msg(aio: *mut nng_aio, msg: *mut nng_msg);
    pub fn nng_aio_get_msg(aio: *mut nng_aio) -> *mut nng_msg;
    pub fn nng_aio_set_timeout(aio: *mut nng_aio, timeout: nng_duration);

    // Message API
    pub fn nng_msg_alloc(msg: &mut *mut nng_msg, cap: usize) -> c_int;
    pub fn nng_msg_free(msg: *mut nng_msg);
    pub fn nng_msg_realloc(msg: *mut nng_msg, size: usize) -> c_int;
    pub fn nng_msg_header(msg: *mut nng_msg) -> *mut c_void;
    pub fn nng_msg_header_len(msg: *const nng_msg) -> usize;
    pub fn nng_msg_body(msg: *mut nng_msg) -> *mut c_void;
    pub fn nng_msg_len(msg: *const nng_msg) -> usize;
    pub fn nng_msg_append(msg: *mut nng_msg, data: *const c_void, sz: usize) -> c_int;
    pub fn nng_msg_insert(msg: *mut nng_msg, data: *const c_void, sz: usize) -> c_int;
    pub fn nng_msg_trim(msg: *mut nng_msg, amount: usize) -> c_int;
    pub fn nng_msg_chop(msg: *mut nng_msg, amount: usize) -> c_int;
    pub fn nng_msg_header_append(msg: *mut nng_msg, buf: *const c_void, amount: usize) -> c_int;
    pub fn nng_msg_header_insert(msg: *mut nng_msg, buf: *const c_void, amount: usize) -> c_int;
    pub fn nng_msg_header_trim(msg: *mut nng_msg, amount: usize) -> c_int;
    pub fn nng_msg_header_chop(msg: *mut nng_msg, amount: usize) -> c_int;
    pub fn nng_msg_header_append_u32(msg: *mut nng_msg, val: u32) -> c_int;
    pub fn nng_msg_header_insert_u32(msg: *mut nng_msg, val: u32) -> c_int;
    pub fn nng_msg_header_chop_u32(msg: *mut nng_msg, valp: &mut u32) -> c_int;
    pub fn nng_msg_header_trim_u32(msg: *mut nng_msg, valp: &mut u32) -> c_int;
    pub fn nng_msg_append_u32(msg: *mut nng_msg, val: u32) -> c_int;
    pub fn nng_msg_insert_u32(msg: *mut nng_msg, val: u32) -> c_int;
    pub fn nng_msg_chop_u32(msg: *mut nng_msg, valp: &mut u32) -> c_int;
    pub fn nng_msg_trim_u32(msg: *mut nng_msg, valp: &mut u32) -> c_int;

    pub fn nng_msg_dup(cpy: &mut *mut nng_msg, msg: *const nng_msg) -> c_int;
    pub fn nng_msg_clear(msg: *mut nng_msg);
    pub fn nng_msg_header_clear(msg: *mut nng_msg);
    pub fn nng_msg_set_pipe(msg: *mut nng_msg, pipe: nng_pipe);
    pub fn nng_msg_get_pipe(msg: *const nng_msg) -> nng_pipe;
    pub fn nng_msg_getopt(msg: *mut nng_msg, opt: c_int, ptr: *mut c_void, sz: &mut usize) -> c_int;

    // Pipe API
    pub fn nng_pipe_getopt(pipe: nng_pipe, opt: *const c_char, valp: *mut c_void, sz: &mut usize) -> c_int;
    pub fn nng_pipe_getopt_int(pipe: nng_pipe, opt: *const c_char, valp: &mut c_int) -> c_int;
    pub fn nng_pipe_getopt_ms(pipe: nng_pipe, opt: *const c_char, valp: &mut nng_duration) -> c_int;
    pub fn nng_pipe_getopt_size(pipe: nng_pipe, opt: *const c_char, valp: &mut usize) -> c_int;
    pub fn nng_pipe_getopt_uint64(pipe: nng_pipe, opt: *const c_char, valp: &mut u64) -> c_int;
    pub fn nng_pipe_close(pipe: nng_pipe) -> c_int;


    // Protocols
    pub fn nng_bus0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_pair0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_pair1_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_push0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_pull0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_pub0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_sub0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_req0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_rep0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_respondent0_open(sock: &mut nng_socket) -> c_int;
    pub fn nng_surveyor0_open(sock: &mut nng_socket) -> c_int;


    pub fn nng_device(sock1: nng_socket, sock2: nng_socket) -> c_int;

    // Statistics
    pub fn nng_snapshot_create(socket: nng_socket, snap: &mut *mut nng_snapshot) -> c_int;
    pub fn nng_snapshot_free(snapshot: *mut nng_snapshot);
    pub fn nng_snapshot_update(snapshot: *mut nng_snapshot) -> c_int;
    pub fn nng_snapshot_next(snapshot: *mut nng_snapshot, stat: &mut *mut nng_stat) -> c_int;
    pub fn nng_stat_name(stat: *mut nng_stat) -> *const c_char;
    pub fn nng_stat_type(stat: *mut nng_stat) -> c_int;

    pub fn nng_stat_unit(stat: *mut nng_stat) -> c_int;

    pub fn nng_stat_value(stat: *mut nng_stat) -> i64;
}
