#![feature(bool_to_option, with_options)]

extern crate fst;
extern crate memmap;
extern crate rhkb_lib;
extern crate structopt;

mod hotkeys;

use std::{
    fs::{DirBuilder, File},
    io::{self, BufWriter},
    path::PathBuf,
};

use hotkeys::{cmd, Builder};

use rhkb_lib::{
    keyboard::{self, french},
    Key::{Press, Release},
    KeyboardInputStream,
};
use structopt::StructOpt;

const BASE_DIR: [&str; 3] = [env!("HOME"), ".config", "rhkb"];

fn bind(ctrl: &mut Builder) {
    ctrl.bind(
        &[french::H, french::J, french::K, french::L],
        cmd("alacritty"),
    );
}

fn main() -> io::Result<()> {
    let parsed = Config::from_args();
    if parsed.log.is_none() || parsed.fst.is_none() {
        build_config_dir()?;
    }

    let mut log = get_logger(parsed.log.as_ref()).transpose()?;
    let socket = parsed
        .keyboard
        .unwrap_or_else(|| "/dev/input/event3".into());

    let fst = parsed.fst.unwrap_or_else(|| {
        let mut base: PathBuf = BASE_DIR.iter().collect();
        base.push("hk.fst");
        base
    });

    let eventstream = KeyboardInputStream::new(socket).expect("couldn't read socket");
    let mut builder = Builder::new(10, parsed.update);
    bind(&mut builder);

    let mut ctrl = builder.finish(fst)?;

    for event in eventstream {
        match event? {
            Press(key) => ctrl.register_press(key),
            Release(key) => ctrl.register_release(key),
            _ => {}
        }
        ctrl.update(log.as_mut())?;
    }
    Ok(())
}

fn build_config_dir() -> io::Result<()> {
    let path: PathBuf = BASE_DIR.iter().collect();
    DirBuilder::new().recursive(true).create(path)
}

fn get_logger(log: Option<&PathBuf>) -> Option<io::Result<BufWriter<File>>> {
    let mut path: PathBuf = BASE_DIR.iter().collect();
    let log = log.unwrap_or_else(|| {
        path.push("rhkd.log");
        &path
    });

    if log.as_os_str() == "no" {
        return None;
    }

    Some(File::create(&path).map(BufWriter::new))
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rhkb", about = "Rust Hotkey Daemon")]
struct Config {
    #[structopt(short, long, help = "rebuilds the hotkey fst at the start")]
    update: bool,

    #[structopt(
        short,
        long,
        parse(from_os_str),
        help = "keyboard socket (requires at least read access)"
    )]
    keyboard: Option<PathBuf>,

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
