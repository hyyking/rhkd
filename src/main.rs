extern crate fst;
extern crate mio;
extern crate signal_hook;
extern crate signal_hook_mio;
extern crate x11;

mod binds;
mod controler;
mod key;
mod keyboard;

use std::{env, io};

use binds::bind;
use controler::Builder;
use keyboard::{DisplayContext, Event, Keyboard};

use mio::{Events, Interest, Poll, Token};
use signal_hook::consts::signal::*;
use signal_hook_mio::v0_7::Signals;

const HELP: &str = "Rust X11 Hotkey Daemon
    --help          Help string
    --fst <PATH>    Path in which to store the fst";

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

const SIGNAL: Token = Token(0);
const KEYBOARD: Token = Token(1);

fn main() -> io::Result<()> {
    let mut poll = Poll::new()?;
    let args = argparse();

    let mut context = DisplayContext::current()?;
    let mut keyboard = Keyboard::new(&mut context);

    let mut builder = Builder::new(&mut keyboard);
    bind(&mut builder);
    let mut ctrl = builder.finish(args.fst.as_deref().unwrap_or("/tmp/rhkb.fst"))?;

    let mut signals = Signals::new(&[SIGTERM, SIGINT])?;
    {
        let registry = poll.registry();
        registry.register(&mut signals, SIGNAL, Interest::READABLE)?;
        registry.register(
            &mut keyboard,
            KEYBOARD,
            Interest::READABLE | Interest::WRITABLE,
        )?;
    }

    let mut events = Events::with_capacity(8);
    loop {
        match poll.poll(&mut events, None) {
            Ok(_) => {}
            Err(a) if a.kind() == io::ErrorKind::Interrupted => {
                continue;
            }
            Err(err) => return Err(err),
        }
        for event in events.iter() {
            match event.token() {
                SIGNAL => return Ok(()),
                KEYBOARD => {
                    keyboard.read_event();
                    match keyboard.decode_event() {
                        Event::KeyPress(key) => ctrl.execute(key),
                        _ => zombie::collect_zombies(),
                    }
                }
                _ => {}
            }
        }
    }
}
