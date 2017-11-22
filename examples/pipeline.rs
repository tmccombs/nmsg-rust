extern crate nmsg;

use std::thread;
use std::time::Duration;
use std::process;

use nmsg::{Push, Pull, SPSocket, SPSend, SPRecv};

const NODE0: &'static str = "node0";
const NODE1: &'static str = "node1";

fn node0(addr: &str) {
    let sock = Pull::new().unwrap();
    sock.bind(addr).unwrap();
    loop {
        let msg = sock.recv().unwrap();
        println!("NODE0: RECEIVED \"{}\"", String::from_utf8_lossy(&msg));
    }
}

fn node1(addr: &str, message: &str) {
    let sock = Push::new().unwrap();
    let endpoint = sock.connect(addr).unwrap();
    println!("NODE1: SENDING \"{}\"", message);
    sock.send_buf(message.as_bytes()).unwrap();

    // Wait for messages to flush.
    thread::sleep(Duration::from_secs(1));
    endpoint.shutdown().unwrap();
}

fn usage() -> ! {
    eprintln!("USAGE: pipeline (node0 <addr>)|(node1 <addr> <url>)");
    process::abort()
}

enum Command {
    Node0(String),
    Node1(String, String)
}

fn parse_args() -> Result<Command, ()> {
    let mut args = std::env::args().skip(1);
    let node = args.next().ok_or(())?;
    let addr = args.next().ok_or(())?;
    if node == NODE0 {
        Ok(Command::Node0(addr))
    } else if node == NODE1 {
        let msg = args.next().ok_or(())?;
        Ok(Command::Node1(addr, msg))
    } else {
        Err(())
    }

}

fn main() {
    match parse_args() {
        Ok(Command::Node0(addr)) => node0(&addr),
        Ok(Command::Node1(addr, msg)) => node1(&addr, &msg),
        _ => usage()
    }
}
