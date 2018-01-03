use socket::Socket;

use nng_sys::*;

pub trait Protocol {
    fn new() -> Self;

    fn socket(&self) -> &Socket;
    fn socket_mut(&mut self) -> &mut Socket;
}

macro_rules! def_protos {
    ($( $(#[$attrs:meta])* struct $name:ident ($open:ident) ;)*) => {$(
        $(#[$attrs])*
        pub struct $name {
            sock: Socket
        }

        impl Protocol for $name {
            fn new() -> $name {
                let mut sock: nng_socket = 0;
                unsafe {
                    assert!($open(&mut sock) > 0);
                    $name {
                        sock: Socket::from_raw(sock)
                    }
                }
            }

            fn socket(&self) -> &Socket {
                &self.sock
            }

            fn socket_mut(&mut self) -> &mut Socket {
                &mut self.sock
            }
        }
    )*}
}

def_protos! {
    struct Bus(nng_bus0_open);
    struct Pair(nng_pair1_open);
    struct Push(nng_push0_open);
    struct Pull(nng_pull0_open);
    struct Pub(nng_pub0_open);
    struct Sub(nng_sub0_open);
    struct Req(nng_req0_open);
    struct Rep(nng_rep0_open);
    struct Respondent(nng_respondent0_open);
    struct Surveyor(nng_surveyor0_open);
}
