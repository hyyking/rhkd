pub mod keyboard;

use std::{
    fs::File,
    io::{self, Read},
    mem,
    path::Path,
};

const KEY_RELEASE: i32 = 0;
const KEY_PRESS: i32 = 1;
const EV_KEY: u16 = 1;

#[derive(Debug)]
pub enum Key {
    Press(u16),
    Release(u16),
    Other,
}

pub struct KeyboardInputStream {
    fd: File,
    buf: [u8; 24],
}

impl KeyboardInputStream {
    /// # Errors
    /// Will throw an error if the file can't be open
    pub fn new<T: AsRef<Path>>(file: T) -> io::Result<Self> {
        let fd = File::open(file)?;
        let buf = [0; 24];
        Ok(Self { fd, buf })
    }
}

#[repr(C)]
struct Event {
    tv_sec: isize,
    tv_usec: isize,
    kind: u16,
    code: u16,
    value: i32,
}

impl Iterator for KeyboardInputStream {
    type Item = io::Result<Key>;

    fn next(&mut self) -> Option<Self::Item> {
        let read = match self.fd.read(&mut self.buf) {
            Ok(read) => read,
            Err(err) => return Some(Err(err)),
        };
        if read != mem::size_of::<Event>() {
            return Some(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "not enough bytes to read event",
            )));
        }

        let event = unsafe { mem::transmute_copy::<_, Event>(&self.buf) };

        let decoded = match (event.kind, event.value) {
            (EV_KEY, KEY_PRESS) => Key::Press(event.code),
            (EV_KEY, KEY_RELEASE) => Key::Release(event.code),
            (_, _) => Key::Other,
        };
        Some(Ok(decoded))
    }
}
