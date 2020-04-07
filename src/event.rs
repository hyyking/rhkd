use std::fs::File;
use std::io;
use std::path::Path;

const KEY_RELEASE: i32 = 0;
const KEY_PRESS: i32 = 1;

const EV_KEY: u16 = 1;

#[derive(Debug)]
pub enum KeyEvent {
    Press(u16),
    Release(u16),
    Pending,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct Event {
    tv_sec: isize,
    tv_usec: isize,
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}

pub struct KeyEventStream {
    fd: File,
    buf: [u8; 24],
}

impl Event {
    #[inline]
    pub fn matches_key(&self, key: u16) -> bool {
        self.code == key
    }
}

impl KeyEventStream {
    pub fn new(file: &Path) -> io::Result<Self> {
        let fd = File::open(file)?;
        let buf = [0; 24];
        Ok(Self { fd, buf })
    }
}

impl Iterator for KeyEventStream {
    type Item = io::Result<KeyEvent>;
    fn next(&mut self) -> Option<Self::Item> {
        use self::KeyEvent::*;
        use std::{io::Read, mem};

        // this should be considerer as closed if we can't read the fd
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
        if event.kind == EV_KEY {
            let var = match event.value {
                KEY_PRESS => Press(event.code),
                KEY_RELEASE => Release(event.code),
                _ => Pending,
            };
            Some(Ok(var))
        } else {
            Some(Ok(Pending))
        }
    }
}
