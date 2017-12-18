use std::io;
use std::option;
use std::net;
use nng_sys::*;

/// An address for a Nanomsg socket.
#[derive(Clone)]
pub enum SocketAddr {
    Unspecified,
    InProc(Vec<u8>),
    Ipc(Vec<u8>),
    Inet(net::SocketAddr),
    ZeroTier {
        nwid: u64,
        nodeid: u64,
        port: u32
    }
}

use std::net::SocketAddr::*;
use self::SocketAddr::*;


impl From<nng_sockaddr> for SocketAddr {
    fn from(sockaddr: nng_sockaddr) -> SocketAddr {
        unsafe {
            match sockaddr.s_family {
                NNG_AF_INPROC => InProc(extract_path(&sockaddr.s_inproc)),
                NNG_AF_IPC => Ipc(extract_path(&sockaddr.s_path)),
                NNG_AF_INET => {
                    let addr = sockaddr.s_in;
                    Inet(V4(net::SocketAddrV4::new(addr.sa_addr.into(), addr.sa_port)))
                },
                NNG_AF_INET6 => {
                    let addr = sockaddr.s_in6;
                    Inet(V6(net::SocketAddrV6::new(addr.sa_addr.into(), addr.sa_port, 0, 0)))
                },
                NNG_AF_ZT => ZeroTier {
                    nwid: sockaddr.s_zt.sa_nwid,
                    nodeid: sockaddr.s_zt.sa_nodeid,
                    port: sockaddr.s_zt.sa_port
                },
                _ => Unspecified
            }
        }
    }
}

impl net::ToSocketAddrs for SocketAddr {
    type Iter = option::IntoIter<net::SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(match *self {
            Inet(a) => Some(a),
            _ => None
        }.into_iter())
    }
}

impl From<net::SocketAddr> for SocketAddr {
    fn from(addr: net::SocketAddr) -> SocketAddr {
        Inet(addr)
    }
}

fn extract_path(addr: &nng_sockaddr_path) -> Vec<u8> {
    let path = &addr.sa_path;
    let end = path.iter().position(|&i| i == 0).unwrap_or(path.len());
    path[..end].to_owned()
}

