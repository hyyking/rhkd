use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::process::{Command, Stdio};

use fst::{Map, MapBuilder};

pub fn cmd<S: AsRef<OsStr>>(c: S) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(c);
    cmd.stdin(Stdio::null());
    cmd
}

type Error = ();

#[derive(Debug, Copy, Clone, Eq, Ord, PartialOrd, PartialEq)]
struct Bind {
    keys_idx: usize,
    keys: [u8; 12],
}
impl Bind {
    fn new() -> Self {
        Self {
            keys_idx: 2,
            keys: [0; 12],
        }
    }
    fn add_mod(&mut self, key: u16) {
        let [first, second] = key.to_ne_bytes();
        self.keys[0] |= first;
        self.keys[1] |= second;
    }
    fn remove_mod(&mut self, key: u16) {
        let [first, second] = key.to_ne_bytes();
        self.keys[0] &= !first;
        self.keys[1] &= !second;
    }
    fn add_key(&mut self, key: u16) -> Result<(), Error> {
        if self.keys_idx == 12 {
            return Err(());
        }
        let [first, second] = key.to_ne_bytes();
        self.keys[self.keys_idx] = first;
        self.keys[self.keys_idx + 1] = second;
        self.keys_idx += 2;
        Ok(())
    }
    fn pop(&mut self) -> Option<u16> {
        if self.keys_idx == 2 {
            return None;
        }

        let order = [self.keys[self.keys_idx - 2], self.keys[self.keys_idx - 1]];
        self.keys[self.keys_idx - 2] = 0;
        self.keys[self.keys_idx - 1] = 0;
        self.keys_idx -= 2;
        Some(u16::from_ne_bytes(order))
    }
    fn is_empty(&self) -> bool {
        self.keys == [0; 12]
    }
}

impl AsRef<[u8]> for Bind {
    fn as_ref(&self) -> &[u8] {
        &self.keys
    }
}

pub struct Builder {
    commands: Vec<Command>,
    binds: Option<Vec<(Bind, u64)>>,
}

// Ok(value) is the mapped control key
// Err(value) is the input
fn is_control_character(key: u16) -> Result<u16, u16> {
    use rhkb_lib::keyboard::{ALT, CTRL, ENTER, ESC, L_SHIFT, MAJ, MOD, R_SHIFT, SPACE, TAB};
    let code = match key {
        ESC => 2,
        CTRL => 2 << 1,
        ENTER => 2 << 2,
        L_SHIFT => 2 << 3,
        R_SHIFT => 2 << 4,
        ALT => 2 << 5,
        MOD => 2 << 6,
        SPACE => 2 << 7,
        TAB => 2 << 8,
        MAJ => 2 << 9,
        key => return Err(key),
    };
    Ok(code)
}

impl Builder {
    pub fn new(cap: usize, reset: bool) -> Self {
        let commands = Vec::with_capacity(cap);
        let binds = reset.then(|| Vec::with_capacity(cap));
        Self { commands, binds }
    }

    pub fn bind<S: IntoIterator<Item = &'static u16>>(
        &mut self,
        pattern: S,
        cmd: Command,
    ) -> &mut Self {
        let index = self.commands.len();
        self.commands.push(cmd);

        if self.binds.is_none() {
            // not in reset mode so no need to keep track of the keys
            return self;
        }
        let binds = self.binds.as_mut().expect("should be set");
        let mut current = Bind::new();

        for key in pattern {
            match is_control_character(*key) {
                Ok(ctrl) => current.add_mod(ctrl),
                Err(key) => current.add_key(key).expect("too many keys"),
            }
        }
        assert!(!current.is_empty(), "can't bind an empty pattern");
        binds.push((current, index as u64));
        self
    }

    pub fn finish<T: AsRef<std::path::Path>>(mut self, path: T) -> std::io::Result<Controler> {
        if let Some(mut binds) = self.binds.take() {
            let file = OpenOptions::new().write(true).open(&path)?;
            let mut map = MapBuilder::new(BufWriter::new(file)).unwrap();
            binds.as_mut_slice().sort_unstable_by_key(|el| el.0);
            for (bind, key) in binds {
                map.insert(bind, key).unwrap();
            }
            map.finish().unwrap();
        }

        let file = File::open(path)?;
        let mm = unsafe { memmap::Mmap::map(&file)? };
        let map = Map::new(mm).expect("couldn't load map");
        Ok(Controler {
            _fd: file,
            current: Bind::new(),
            cmds: self.commands,
            map,
        })
    }
}

pub struct Controler {
    _fd: File,
    current: Bind,
    cmds: Vec<Command>,
    map: Map<memmap::Mmap>,
}

impl Controler {
    pub fn update<T: std::io::Write>(&mut self, log: &mut T) {
        if let Some(idx) = self.map.get(self.current) {
            writeln!(log, "{:?}", self.cmds[idx as usize].output()).expect("couldn't write")
        }
    }

    pub fn register_press(&mut self, key: u16) {
        match is_control_character(key) {
            Ok(ctrl) => self.current.add_mod(ctrl),
            Err(key) => self.current.add_key(key).expect("buffer overflow"),
        }
    }
    pub fn register_release(&mut self, key: u16) {
        if let Ok(ctrl) = is_control_character(key) {
            self.current.remove_mod(ctrl);
        } else {
            let _ = self.current.pop();
        }
    }
}
