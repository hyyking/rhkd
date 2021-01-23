use std::{
    fs::OpenOptions,
    io::{self, BufWriter},
    str::FromStr,
};

use crate::{
    key::{self, Cmd, Key, Locks},
    keyboard::Keyboard,
};

use fst::{self, Map, MapBuilder};

pub struct Controler {
    cmds: Box<[Cmd]>,
    map: Map<memmap::Mmap>,
}

pub struct Builder<'a, 'kb> {
    commands: Vec<Cmd>,
    binds: Vec<([u8; 16], u64)>,
    locks: Locks,
    keyboard: &'a mut Keyboard<'kb>,
}

impl<'a, 'kb> Builder<'a, 'kb> {
    pub fn new(keyboard: &'a mut Keyboard<'kb>) -> Self {
        Self {
            commands: Vec::new(),
            binds: Vec::new(),
            locks: Locks::new(),
            keyboard,
        }
    }
    pub fn bind(&mut self, pattern: &str, cmd: &str) {
        self.try_bind(pattern, cmd).expect("Unable to bind key");
    }
    pub fn try_bind(&mut self, pattern: &str, cmd: &str) -> Result<(), key::Error> {
        let key = Key::from_str(pattern)?;
        let cmd = Cmd::from_str(cmd)?;
        let Locks { num, caps } = self.locks;

        let numlocked = key.merge(Key::mask(num.unwrap_or(0)));
        let capslocked = key.merge(Key::mask(caps.unwrap_or(x11::xlib::LockMask)));
        let all_locked = numlocked.merge(capslocked);

        self.keyboard.grab_key(key).map_err(|_| ())?;
        self.keyboard.grab_key(numlocked).map_err(|_| ())?;
        self.keyboard.grab_key(capslocked).map_err(|_| ())?;
        self.keyboard.grab_key(all_locked).map_err(|_| ())?;

        let idx = self.commands.len() as u64;
        self.commands.push(cmd);
        self.binds.push((key.into(), idx));
        self.binds.push((numlocked.into(), idx));
        self.binds.push((capslocked.into(), idx));
        self.binds.push((all_locked.into(), idx));
        Ok(())
    }

    pub fn finish<T: AsRef<std::path::Path>>(mut self, path: T) -> io::Result<Controler> {
        self.commands.shrink_to_fit();
        self.binds.sort_unstable_by_key(|k| k.0);

        let cmds = self.commands.into_boxed_slice();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let mut b = MapBuilder::new(BufWriter::new(file)).map_err(fsterror_to_io)?;
        self.binds
            .drain(..)
            .for_each(|(key, entry)| b.insert(key, entry).unwrap());

        let file = b
            .into_inner()
            .map_err(fsterror_to_io)?
            .into_inner()
            .expect("issue with the inner bufwriter");

        let map = Map::new(unsafe { memmap::Mmap::map(&file)? }).map_err(fsterror_to_io)?;
        Ok(Controler { cmds, map })
    }
}

impl Controler {
    pub fn execute(&mut self, key: Key) {
        use std::convert::TryInto;
        if let Some(index) = self.map.get::<[u8; 16]>(key.into()) {
            if usize::max_value()
                .try_into()
                .map(|v| index > v)
                .unwrap_or(true)
            {
                return;
            }
            if let Err(err) = self.cmds[index as usize].0.spawn() {
                eprintln!("command failed to spawn {:?}", err);
            }
        }
    }
}

fn fsterror_to_io(err: fst::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, err)
}
