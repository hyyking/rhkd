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

use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use binds::bind;
use controler::Builder;
use event::{signal::SigHandler, Event::KeyPress, Keyboard};

use libc::{SIGINT, SIGTERM};
use structopt::StructOpt;

static RUN: AtomicBool = AtomicBool::new(true);

fn main() -> io::Result<()> {
    let parsed = Config::from_args();

    let fst = parsed.fst.unwrap_or_else(|| PathBuf::from("/tmp/rhkb.fst"));

    let mut eventstream = Keyboard::new().expect("couldn't connect to X11 server");
    let mut builder = Builder::new(eventstream.context()?);
    bind(&mut builder);
    let mut ctrl = builder.finish(&fst)?;

    let hd = SigHandler::new(|code, _, _| {
        if matches!(code, SIGTERM | SIGINT) {
            RUN.store(false, Ordering::SeqCst);
        }
    });
    hd.register(SIGTERM)?;
    hd.register(SIGINT)?;

    while RUN.load(Ordering::SeqCst) {
        match eventstream.poll() {
            Poll::Ready(KeyPress(key)) => ctrl.execute(key),
            Poll::Ready(_) | Poll::Pending => continue,
        }
    }
    Ok(())
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
