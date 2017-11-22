extern crate nmsg;

use std::env::args;
use std::thread::sleep;
use std::time::Duration;

use nmsg::*;

fn send_name(sock: &Pair, name: &str) -> Result<()> {
    println!("{}: SENDING \"{}\"", name, name);
    sock.send_buf(name.as_bytes())?;
    Ok(())
}

fn recv_name(sock: &Pair, name: &str) -> Result<()> {
    let msg = sock.recv()?;
    println!("{}: RECEIVED \"{}\"", name, String::from_utf8_lossy(&msg));
    Ok(())
}

fn send_recv(sock: Pair, name: &str) -> Result<()> {
    sock.set_rcv_timeout(1000)?;
    loop {
        send_name(&sock, name)?;
        sleep(Duration::from_secs(1));
        recv_name(&sock, name)?;
    }
}

fn usage() -> ! {
    eprintln!("USAGE: pair NAME URL");
    std::process::abort();
}

fn parse_args() -> Option<(String, String)> {
    let mut args = args().skip(1);
    let name = match args.next() {
        Some(s) => s,
        None => return None
    };
    let url = match args.next() {
        Some(s) => s,
        None => return None
    };
    Some((name, url))
}

fn run(name: &str, url: &str) -> Result<()> {
        let sock = Pair::new()?;
        let endpoint = if name == "node0" {
            sock.bind(&url)?
        } else {
            sock.connect(&url)?
        };
        send_recv(sock, name)?;
        sleep(Duration::from_secs(1));
        endpoint.shutdown()
}

fn main() {
    if let Some((name, url)) = parse_args() {
        if let Err(e) = run(&name, &url) {
            println!("ERROR: {}", e);
        }
    } else {
        usage();
    }
}
