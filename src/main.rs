#![feature(bool_to_option, with_options, never_type)]

extern crate fst;
extern crate libc;
extern crate memmap;
extern crate x11;
extern crate zombie;

mod binds;
mod controler;
mod event;
mod key;

use std::{
    env, io,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use binds::bind;
use controler::Builder;
use event::{signal::SigHandler, Event::KeyPress, Keyboard};

use libc::{SIGINT, SIGTERM};

const HELP: &str = "Rust X11 Hotkey Daemon
    --help          help string
    --fst <PATH>     Path in which to store the fst";

fn exit() -> ! {
    eprintln!("{}", HELP);
    std::process::exit(1)
}

#[derive(Default)]
struct Args {
    fst: Option<String>,
}

fn argparse() -> Args {
    let mut args = env::args().skip(1);
    let mut output = Args::default();

    match args.len() {
        0 => {}
        1 => {
            if let Some("--help") = args.next().as_deref() {
                println!("{}", HELP);
                std::process::exit(0)
            } else {
                exit()
            }
        }
        a if a % 2 == 0 => {
            for _ in (0..a).step_by(2) {
                match args.next().as_deref() {
                    Some("--fst") => output.fst = args.next(),
                    Some(_) | None => exit(),
                }
            }
        }
        _ => exit(),
    }
    output
}

static RUN: AtomicBool = AtomicBool::new(true);

fn main() -> io::Result<()> {
    let args = argparse();

    let mut eventstream = Keyboard::new().expect("couldn't connect to X11 server");
    let mut builder = Builder::new(eventstream.context()?);
    bind(&mut builder);

    let mut ctrl = builder.finish(args.fst.as_deref().unwrap_or("/tmp/rhkb.fst"))?;

    let hd = SigHandler::new(|code, _, _| {
        if matches!(code, SIGTERM | SIGINT) {
            RUN.store(false, Ordering::SeqCst);
        }
    });
    hd.register(SIGTERM)?;
    hd.register(SIGINT)?;

    while RUN.load(Ordering::SeqCst) {
        if let Poll::Ready(KeyPress(key)) = eventstream.poll() {
            zombie::collect_zombies();
            ctrl.execute(key)
        }
    }
    Ok(())
}
