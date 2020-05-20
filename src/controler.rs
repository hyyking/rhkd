use std::{
    fs::File,
    io::{self, BufWriter},
    str::FromStr,
};

use crate::{
    event::GrabContext,
    key::{Cmd, Key, Locks},
};

use fst::{self, Map, MapBuilder};

pub struct Controler<'a> {
    cmds: Box<[Cmd]>,
    map: Map<memmap::Mmap>,
    _grab: GrabContext<'a>, // ungrabs the keys on drop
}

pub struct Builder<'a> {
    commands: Vec<Cmd>,
    binds: Vec<([u8; 12], u64)>,
    locks: Locks,
    grab: GrabContext<'a>,
}

impl<'a> Builder<'a> {
    pub fn new(grab: GrabContext<'a>) -> Self {
        Self {
            commands: Vec::new(),
            binds: Vec::new(),
            locks: Locks::new(),
            grab,
        }
    }
    pub fn bind(&mut self, pattern: &str, cmd: &str) {
        let key = Key::from_str(pattern).expect("unable to parse Key from str");
        let cmd = Cmd::from_str(cmd).expect("unbale to parse Cmd from str");
        let Locks { num, caps } = self.locks;

        let numlocked = {
            let mut key = key;
            key.mask |= num.unwrap_or(0);
            key
        };
        let capslocked = {
            let mut key = key;
            key.mask |= caps.unwrap_or(x11::xlib::LockMask);
            key
        };
        let all_locked = {
            let mut key = key;
            key.mask |= caps.unwrap_or(x11::xlib::LockMask) | num.unwrap_or(0);
            key
        };

        self.grab.grab_key(key).expect("unable to grab key");
        self.grab.grab_key(numlocked).expect("unable to grab key");
        self.grab.grab_key(capslocked).expect("unable to grab key");
        self.grab.grab_key(all_locked).expect("unable to grab key");

        let idx = self.commands.len() as u64;
        self.commands.push(cmd);
        self.binds.push((key.into(), idx));
        self.binds.push((numlocked.into(), idx));
        self.binds.push((capslocked.into(), idx));
        self.binds.push((all_locked.into(), idx));
    }

    pub fn finish<T: AsRef<std::path::Path>>(mut self, path: T) -> io::Result<Controler<'a>> {
        self.commands.shrink_to_fit();
        self.binds.sort_unstable_by_key(|k| k.0);

        let cmds = self.commands.into_boxed_slice();

        let file = File::with_options()
            .read(true)
            .write(true)
            .create(true)
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
        Ok(Controler {
            cmds,
            map,
            _grab: self.grab,
        })
    }
}

impl<'a> Controler<'a> {
    pub fn execute(&mut self, key: Key) {
        use std::convert::TryInto;
        if let Some(index) = self.map.get::<[u8; 12]>(key.into()) {
            if index > usize::max_value().try_into().unwrap() {
                return;
            }
            let command = &mut self.cmds[index as usize].0;
            if let Ok(mut child) = command.spawn() {
                let _ = child.wait();
            } else {
                eprintln!("command {:?} didn't start", &command);
            }
        }
    }
}

fn fsterror_to_io(err: fst::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, err)
}
