#![feature(bool_to_option, with_options)]

extern crate fst;
extern crate libc;
extern crate memmap;
extern crate structopt;
extern crate x11;

mod controler;
mod fddriver;
mod key;
mod listener;

use std::{
    fs::File,
    io::{self, BufWriter},
    path::{Path, PathBuf},
    task::Poll,
};

use controler::Builder;

use listener::{Event::KeyPress, Keyboard};

use structopt::StructOpt;

fn bind(b: &mut Builder) {
    use std::process::Command;

    b.bind("ctrl + shift + u", Command::new("ls"));
}

fn main() -> io::Result<()> {
    let parsed = Config::from_args();

    let _log =
        get_logger(&parsed.log.unwrap_or_else(|| PathBuf::from("/tmp/rhkb.log"))).transpose()?;

    let fst = parsed.fst.unwrap_or_else(|| PathBuf::from("/tmp/rhkb.fst"));

    let mut eventstream = Keyboard::new().expect("couldn't connect to X11 server");
    let mut builder = Builder::new(eventstream.context()?, 40);
    bind(&mut builder);
    let mut ctrl = builder.finish(fst)?;

    loop {
        match eventstream.poll() {
            Poll::Ready(KeyPress(key)) => ctrl.execute(key),
            Poll::Ready(_) | Poll::Pending => {}
        }
        if false {
            break;
        }
    }
    Ok(())
}

fn get_logger(log: &Path) -> Option<io::Result<BufWriter<File>>> {
    if log.as_os_str() == "no" {
        return None;
    }

    Some(File::create(log).map(BufWriter::new))
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rhkb", about = "Rust Hotkey Daemon")]
struct Config {
    #[structopt(
        short,
        long,
        parse(from_os_str),
        help = "directory in which to store the fst"
    )]
    fst: Option<PathBuf>,

    #[structopt(long, parse(from_os_str), help = "path of the loging file")]
    log: Option<PathBuf>,
}
