mod controller;
mod event;
mod keys;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
// use std::process::Command;

use controller::{cmd, KeyboardState};
use event::{KeyEvent::*, KeyEventStream};

fn bind(ctrl: &mut KeyboardState) {
    ctrl.bind(&[keys::CTRL, keys::ALT, keys::F], cmd("echo hello world"));
    ctrl.init();
}

fn main() {
    let mut log = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("rhkd.log")
        .unwrap();

    let eventstream = KeyEventStream::new(Path::new("/dev/input/event3")).unwrap();
    let mut ctrl = KeyboardState::new(5);
    bind(&mut ctrl);

    for event in eventstream {
        match event {
            Ok(Press(key)) => {
                ctrl.register_press(key);
            }
            Ok(Release(key)) => ctrl.register_release(key),
            Ok(Pending) => {}
            Err(e) => panic!(e),
        }
        ctrl.update(&mut log);
    }
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
