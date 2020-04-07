use std::{
    cmp::Ordering,
    ffi::OsStr,
    fs::File,
    io::BufWriter,
    process::{Command, Stdio},
};

use fst::{Map, MapBuilder};

pub fn cmd<S: AsRef<OsStr>>(c: S) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(c);
    cmd.stdin(Stdio::null());
    cmd
}

type Error = ();

#[derive(Debug, Copy, Clone)]
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
impl Eq for Bind {}

impl Ord for Bind {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Bind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let zipped = self.keys.iter().zip(other.keys.iter());
        Some(zipped.fold(Ordering::Equal, |b, (s, o)| b.then(s.cmp(o))))
    }
}

impl PartialEq for Bind {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys
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
        let file = File::with_options()
            .read(true)
            .write(self.binds.is_some()) // will get write access if we need to update
            .open(path)?;

        if let Some(mut binds) = self.binds.take() {
            let mut map = MapBuilder::new(BufWriter::new(&file)).unwrap();
            binds.as_mut_slice().sort_unstable_by_key(|el| el.0);
            for (bind, key) in binds {
                map.insert(bind, key).unwrap();
            }
            map.finish().unwrap();
        }

        let mm = unsafe { memmap::Mmap::map(&file)? };
        let map = Map::new(mm).expect("couldn't load map");
        Ok(Controler {
            inside: false,
            current: Bind::new(),
            cmds: self.commands.into_boxed_slice(),
            map,
        })
    }
}

pub struct Controler {
    inside: bool,
    current: Bind,
    cmds: Box<[Command]>,
    map: Map<memmap::Mmap>,
}

impl Controler {
    pub fn update<T: std::io::Write>(&mut self, log: Option<&mut T>) -> std::io::Result<()> {
        if self.current.is_empty() {
            self.inside = false;
        }

        if self.inside {
            return Ok(());
        }

        let output = self.map.get(self.current).map(|idx| {
            self.inside = true;

            #[allow(clippy::cast_possible_truncation)] // we know this is a usize
            self.cmds[idx as usize].spawn()
        });

        if let (Some(ref output), Some(ref mut logger)) = (output.transpose()?, log) {
            writeln!(logger, "spawned program {:?}", output.id())
        } else {
            Ok(())
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
