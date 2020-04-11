#![feature(bool_to_option, with_options)]

extern crate fst;
extern crate memmap;
extern crate rhkb_lib;
extern crate structopt;

// mod binds;
mod hotkeys;

use std::{
    fs::{DirBuilder, File},
    io::{self, BufWriter},
    path::{Path, PathBuf},
    task::Poll,
};

// use binds::bind;
use hotkeys::Builder;

use rhkb_lib::{
    Event::{KeyPress, KeyRelease},
    KeyboardInputStream,
};

use structopt::StructOpt;

const BASE_DIR: [&str; 3] = [env!("HOME"), ".config", "rhkb"];

fn bind(b: &mut Builder) {
    use rhkb_lib::keyboard::mask::ANY;
    use std::process::Command;

    b.bind(&[ANY], "u", Command::new("ls"));
}

fn main() -> io::Result<()> {
    let parsed = Config::from_args();

    let mut log =
        get_logger(&parsed.log.unwrap_or_else(|| PathBuf::from("/tmp/rhkb.log"))).transpose()?;

    let fst = parsed.fst.unwrap_or_else(|| PathBuf::from("/tmp/rhkb.fst"));

    let mut eventstream = KeyboardInputStream::new().expect("couldn't connect to X11 server");
    let mut builder = Builder::new(eventstream.grabber()?, 40);
    bind(&mut builder);
    let mut ctrl = builder.finish(fst)?;

    loop {
        match eventstream.poll() {
            Poll::Ready(KeyPress(key)) => ctrl.execute(key),
            Poll::Ready(KeyRelease(_)) => {}
            Poll::Ready(_) => {}
            Poll::Pending => {}
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
