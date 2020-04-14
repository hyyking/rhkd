#![feature(bool_to_option, with_options)]

extern crate fst;
extern crate libc;
extern crate memmap;
extern crate structopt;
extern crate x11;

mod binds;
mod controler;
mod event;
mod key;

use std::{io, path::PathBuf, task::Poll};

use binds::bind;
use controler::Builder;
use event::{signal::SigHandler, Event::KeyPress, Keyboard};

use libc::{c_void, siginfo_t as SigInfo, SIGINT, SIGTERM};
use structopt::StructOpt;

#[no_mangle]
fn handler(code: i32, _info: SigInfo, __: *mut c_void) {
    match code {
        SIGTERM => {
            dbg!("SIGTERM");
        }
        SIGINT => {
            dbg!("SIGINT");
        }
        _ => {
            dbg!(code);
        }
    }
}

fn main() -> io::Result<()> {
    let parsed = Config::from_args();

    let fst = parsed.fst.unwrap_or_else(|| PathBuf::from("/tmp/rhkb.fst"));

    let mut eventstream = Keyboard::new().expect("couldn't connect to X11 server");
    let mut builder = Builder::new(eventstream.context()?);
    bind(&mut builder);
    let mut ctrl = builder.finish(&fst)?;

    // let mut hd = SigHandler::new(handler);
    let mut hd = SigHandler::new(|code, _, _| match code {
        SIGTERM => {
            dbg!("SIGTERM");
        }
        SIGINT => {
            dbg!("SIGINT");
        }
        _ => unreachable!(),
    });
    hd.register(SIGTERM)?;
    hd.register(SIGINT)?;

    loop {
        match eventstream.poll() {
            Poll::Ready(KeyPress(key)) => ctrl.execute(key),
            Poll::Ready(_) => continue,
            Poll::Pending => {}
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rhkb", about = "Rust X11 Hotkey Daemon")]
struct Config {
    #[structopt(
        short,
        long,
        parse(from_os_str),
        help = "path in which to store the fst, default is /tmp/"
    )]
    fst: Option<PathBuf>,
}
