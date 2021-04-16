extern crate fst;
extern crate mio;
extern crate signal_hook;
extern crate signal_hook_mio;
extern crate x11;

#[macro_use]
extern crate log;

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

    while let Some(arg) = args.next() {
        match &*arg {
            "--help" => {
                eprintln!("{}", HELP);
                std::process::exit(1)
            }
            "--fst" => output.fst = args.next().ok_or_else(exit).ok(),
            _ => exit(),
        }
    }
    output
}

const SIGNAL: Token = Token(0);
const KEYBOARD: Token = Token(1);

fn main() -> io::Result<()> {
    pretty_env_logger::init_timed();

    let mut poll = Poll::new()?;
    let args = argparse();

    let mut context = DisplayContext::current().unwrap();
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

    let mut events = Events::with_capacity(32);
    let mut xevents = Vec::with_capacity(32);
    let mut collect = 0;
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
                SIGNAL => {
                    trace!("signal event");
                    return Ok(());
                }
                KEYBOARD => {
                    trace!("keyboard event");
                    collect += 1;

                    keyboard.read_events(&mut xevents);
                    xevents.drain(..).for_each(|event| {
                        if let Event::KeyPress(key) = event {
                            ctrl.execute(key);
                        }
                    });

                    if collect >= 10 {
                        info!("collecting zombies");
                        collect = 0;
                        zombie::collect_zombies();
                    }
                }
                _ => {}
            }
        }
    }
}
