extern crate nmsg;

use nmsg::*;

const PREFIX: &'static [u8] = b"Hello, ";

fn server(url: &str) -> Result<()> {
    let sock = Rep::new()?;
    sock.bind(url)?;
    let err = sock.reply_loop(&|req| {
        let offset = PREFIX.len();
        let mut rep = MessageBuffer::new(offset + req.len());
        rep[0..offset].copy_from_slice(PREFIX);
        rep[offset..].copy_from_slice(&req);
        Ok(rep)
    });
    Err(err)
}

fn client(url: &str, body: &str) -> Result<()> {
    let sock = Req::new()?;
    sock.connect(url)?;
    let rep = sock.request(body.into())?;
    println!("REPLY: {}", String::from_utf8_lossy(&rep));
    Ok(())
}

fn usage() -> ! {
    eprintln!("USAGE: {{server URL}} | {{client URL MESSAGE}}");
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
        let msg = try_usage!(args.next());
        client(&url, &msg)
    }
}

fn main() {
    match run() {
        Err(e) => eprintln!("ERROR: {}", e),
        Ok(_) => {}
    }
}
