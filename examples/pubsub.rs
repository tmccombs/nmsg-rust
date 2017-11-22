extern crate nmsg;

use std::borrow::Cow;
use std::mem;
use std::slice;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;

use nmsg::*;

const TIME: &'static str = "TM:";
const COUNT: &'static str = "CT:";

fn epoch() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn write_u64(out: &mut [u8], v: u64) {
    let bytes: &[u8] = unsafe {
        slice::from_raw_parts(&v as *const _ as *const u8, mem::size_of::<u64>())
    };
    out.copy_from_slice(bytes);
}

fn read_msg(msg: &[u8]) -> (Cow<str>, u64) {
    let split = msg.iter().position(|&x| x == b':').unwrap_or(0);
    let prefix = String::from_utf8_lossy(&msg[0..split]);
    let val = if msg.len() >= split + 1 + mem::size_of::<u64>() {
        unsafe { *(&msg[split+1] as *const _ as *const u64) }
    } else {
        0
    };
    (prefix, val)
}

fn publish(sock: &Pub, topic: &str, val: u64) -> Result<()> {
    let offset = topic.len();
    let mut msg = MessageBuffer::new(offset + mem::size_of::<u64>());
    msg[0..offset].copy_from_slice(topic.as_bytes());
    write_u64(&mut msg[offset..], val);
    sock.send(msg)?;
    Ok(())
}

fn server(url: &str) -> Result<()> {
    let mut count: u64 = 0;
    let sock = Pub::new()?;
    sock.bind(url)?;
    loop {
        let time = epoch();
        println!("SERVER PUBLISHING: time={}, count={}", time, count);
        publish(&sock, TIME, time)?;
        publish(&sock, COUNT, count)?;
        count += 1;
        thread::sleep(Duration::from_secs(1));
    }
}

fn client(url: &str, topic: &str) -> Result<()> {
    let sock = Sub::new()?;
    sock.connect(url)?;
    sock.subscribe(topic.as_bytes());
    loop {
        let msg = sock.recv()?;
        let (prefix, val) = read_msg(&msg);
        println!("CLIENT RECEIVED: {} {}", prefix, val);
    }
}

fn usage() -> ! {
    eprintln!("USAGE: pubsub {{server URL}} | {{client URL TOPIC}}");
    std::process::exit(1)
}

macro_rules! try_usage {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => usage()
        }
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let node = try_usage!(args.next());
    let url = try_usage!(args.next());
    if node == "server" {
        server(&url)
    } else {
        let topic = try_usage!(args.next());
        client(&url, &topic)
    }
}


fn main() {
    match run() {
        Err(e) => eprintln!("ERROR: {}", e),
        Ok(_) => {}
    }
}
