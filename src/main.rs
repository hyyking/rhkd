#![feature(bool_to_option, with_options)]

extern crate fst;
extern crate libc;
extern crate memmap;
extern crate x11;

mod binds;
mod controler;
mod event;
mod key;

use std::{
    env,
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use binds::bind;
use controler::Builder;
use event::{signal::SigHandler, Event::KeyPress, Keyboard};

use libc::{SIGINT, SIGTERM};

const HELP: &str = "Rust X11 Hotkey Daemon
    --help          help string
    --fst <ARG>     Path in which to store the fst";

static RUN: AtomicBool = AtomicBool::new(true);

fn exit(mut lock: io::StdoutLock) -> ! {
    lock.write(HELP.as_bytes()).unwrap();
    std::process::exit(1)
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut outlock = io::Stdout::lock(&stdout);

    let mut fst = None;
    let mut args = env::args().skip(1);

    match args.len() {
        0 => {}
        1 => {
            if let Some("--help") = args.next().as_deref() {
                outlock.write(HELP.as_bytes())?;
                return Ok(());
            } else {
                exit(outlock)
            }
        }
        a if a % 2 == 0 => {
            for _ in (0..a).step_by(2) {
                if let Some("--fst") = args.next().as_deref() {
                    fst = args.next();
                } else {
                    exit(outlock)
                }
            }
        }
        _ => exit(outlock),
    }

    let mut eventstream = Keyboard::new().expect("couldn't connect to X11 server");

    let mut builder = Builder::new(eventstream.context()?);
    bind(&mut builder);
    let mut ctrl = builder.finish(fst.unwrap_or_else(|| "/tmp/rhkb.fst".into()).as_str())?;

    let hd = SigHandler::new(|code, _, _| {
        if matches!(code, SIGTERM | SIGINT) {
            RUN.store(false, Ordering::SeqCst);
        }
    });
    hd.register(SIGTERM)?;
    hd.register(SIGINT)?;

    while RUN.load(Ordering::SeqCst) {
        if let Poll::Ready(KeyPress(key)) = eventstream.poll() {
            ctrl.execute(key)
        }
    }
    Ok(())
}
