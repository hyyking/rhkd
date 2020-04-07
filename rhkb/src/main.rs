#![feature(bool_to_option)]

extern crate fst;
extern crate memmap;
extern crate rhkb_lib;

mod hotkeys;

use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::{self, BufWriter},
};

use hotkeys::{cmd, Builder};

use rhkb_lib::{
    keyboard::{self, french},
    Key::{Press, Release},
    KeyboardInputStream,
};

const BASE_DIR: [&str; 3] = [env!("HOME"), ".config", "rhkb"];

fn bind(ctrl: &mut Builder) {
    ctrl.bind(
        &[keyboard::CTRL, keyboard::ALT, french::F],
        cmd("echo hello world"),
    );
}

fn main() -> io::Result<()> {
    let mut log = directory_setup()?;

    let eventstream = KeyboardInputStream::new("/dev/input/event3").unwrap();
    let mut builder = Builder::new(10, true);
    bind(&mut builder);

    let mut ctrl = builder.finish("/home/hyyking/.config/rhkb/keys.fst")?;

    for event in eventstream {
        match event? {
            Press(key) => ctrl.register_press(key),
            Release(key) => ctrl.register_release(key),
            _ => {}
        }
        ctrl.update(&mut log);
    }
    Ok(())
}

fn directory_setup() -> io::Result<BufWriter<File>> {
    use std::path::PathBuf;

    let mut path: PathBuf = BASE_DIR.iter().collect();
    path.push("rhkd.log");

    let _ = DirBuilder::new()
        .recursive(true)
        .create(path.parent().unwrap())?;

    let _ = fs::remove_file(&path);

    let file = OpenOptions::new().create(true).append(true).open(&path)?;
    Ok(BufWriter::new(file))
}

/*
#[derive(Debug)]
struct Config {

}

fn parse_args() -> Config {
    let device_file = get_default_device();
    let log_file = "keys.log".to_owned();

    Config {
        device_file,
        log_file,
    }
}

fn get_default_device() -> String {
    let mut filenames = get_keyboard_device_filenames();

    if filenames.len() == 1 {
        filenames.swap_remove(0)
    } else {
        panic!(
            "The following keyboard devices were detected: {:?}. Please select one using \
                the `-d` flag",
            filenames
        );
    }
}

// Detects and returns the name of the keyboard device file. This function uses
// the fact that all device information is shown in /proc/bus/input/devices and
// the keyboard device file should always have an EV of 120013
fn get_keyboard_device_filenames() -> Vec<String> {
    let mut command_str = "grep -E 'Handlers|EV' /proc/bus/input/devices".to_string();
    command_str.push_str("| grep -B1 120013");
    command_str.push_str("| grep -Eo event[0-9]+");

    let res = Command::new("sh")
        .arg("-c")
        .arg(command_str)
        .output()
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
    let res_str = std::str::from_utf8(&res.stdout).unwrap();

    let mut filenames = Vec::new();
    for file in res_str.trim().split('\n') {
        let mut filename = "/dev/input/".to_string();
        filename.push_str(file);
        filenames.push(filename);
    }
    filenames
}
*/
