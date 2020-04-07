use std::ffi::OsStr;
use std::process::{Command, Stdio};

use crate::keys;

pub fn cmd<S: AsRef<OsStr>>(c: S) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(c);
    cmd.stdin(Stdio::null());
    cmd
}

#[derive(Copy, Clone, Eq, Ord, PartialOrd, PartialEq)]
struct Bind {
    modifiers: u16,
    keys: u16,
}

pub struct KeyboardState {
    modifiers: Option<u16>,
    keys: Option<u16>,
    binds: Vec<(Bind, Command)>,
}

impl KeyboardState {
    pub fn new(capacity: usize) -> Self {
        Self {
            modifiers: None,
            keys: None,
            binds: Vec::with_capacity(capacity),
        }
    }
    pub fn bind<S: IntoIterator<Item = &'static u16>>(&mut self, pattern: S, cmd: Command) {
        let mut m = 0;
        let mut keys = 0;
        for key in pattern {
            match *key {
                keys::ESC => m |= 2,
                keys::CTRL => m |= 2 << 1,
                keys::ENTER => m |= 2 << 2,
                keys::L_SHIFT => m |= 2 << 3,
                keys::R_SHIFT => m |= 2 << 4,
                keys::ALT => m |= 2 << 5,
                keys::MOD => m |= 2 << 6,
                keys::SPACE => m |= 2 << 7,
                keys::TAB => m |= 2 << 8,
                keys::MAJ => m |= 2 << 9,
                key => {
                    if keys == 0 {
                        keys = key;
                    } else {
                        panic!("the key {} is already bound", key);
                    }
                }
            }
        }
        if m == 0 || keys == 0 {
            panic!("invalid pattern")
        }
        let b = Bind { modifiers: m, keys };
        self.binds.push((b, cmd));
    }
    pub fn init(&mut self) {
        self.binds.as_mut_slice().sort_unstable_by_key(|el| el.0);
    }
    pub fn update<T: std::io::Write>(&mut self, log: &mut T) {
        if let (Some(modifiers), Some(keys)) = (self.modifiers, self.keys) {
            let bind = Bind { modifiers, keys };
            if let Ok(idx) = self.binds.as_slice().binary_search_by_key(&bind, |el| el.0) {
                assert!(write!(log, "{:#?}\n", self.binds[idx].1.output()).is_ok());
            }
        }
    }

    pub fn register_press(&mut self, key: u16) {
        match key {
            keys::ESC => self.modifiers = Some(self.modifiers.map_or(2, |m| m | 2)),
            keys::CTRL => self.modifiers = Some(self.modifiers.map_or(2 << 1, |m| m | 2 << 1)),
            keys::ENTER => self.modifiers = Some(self.modifiers.map_or(2 << 2, |m| m | 2 << 2)),
            keys::L_SHIFT => self.modifiers = Some(self.modifiers.map_or(2 << 3, |m| m | 2 << 3)),
            keys::R_SHIFT => self.modifiers = Some(self.modifiers.map_or(2 << 4, |m| m | 2 << 4)),
            keys::ALT => self.modifiers = Some(self.modifiers.map_or(2 << 5, |m| m | 2 << 5)),
            keys::MOD => self.modifiers = Some(self.modifiers.map_or(2 << 6, |m| m | 2 << 6)),
            keys::SPACE => self.modifiers = Some(self.modifiers.map_or(2 << 7, |m| m | 2 << 7)),
            keys::TAB => self.modifiers = Some(self.modifiers.map_or(2 << 8, |m| m | 2 << 8)),
            keys::MAJ => self.modifiers = Some(self.modifiers.map_or(2 << 9, |m| m | 2 << 9)),
            _ => self.keys = Some(key),
        }
    }
    pub fn register_release(&mut self, key: u16) {
        match key {
            keys::ESC => self.modifiers = self.modifiers.map(|m| m & 2),
            keys::CTRL => self.modifiers = self.modifiers.map(|m| m & (2 << 1)),
            keys::ENTER => self.modifiers = self.modifiers.map(|m| m & (2 << 2)),
            keys::L_SHIFT => self.modifiers = self.modifiers.map(|m| m & 2 << 3),
            keys::R_SHIFT => self.modifiers = self.modifiers.map(|m| m & 2 << 4),
            keys::ALT => self.modifiers = self.modifiers.map(|m| m & 2 << 5),
            keys::MOD => self.modifiers = self.modifiers.map(|m| m & 2 << 6),
            keys::SPACE => self.modifiers = self.modifiers.map(|m| m & 2 << 7),
            keys::TAB => self.modifiers = self.modifiers.map(|m| m & 2 << 8),
            keys::MAJ => self.modifiers = self.modifiers.map(|m| m & 2 << 9),
            _ => self.keys = None,
        }
        if self.modifiers == Some(0) {
            self.modifiers = None;
        }
    }
}
