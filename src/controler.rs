use std::{
    alloc::Layout,
    convert::TryInto,
    fs::OpenOptions,
    io::{self, BufWriter},
    path::Path,
    str::FromStr,
};

use crate::{
    exec::{Exec, IntoExec},
    key::{self, Key, Locks},
    keyboard::Keyboard,
};

use fst::{self, Map, MapBuilder};

pub struct Controler {
    cmds: Box<[Exec]>,
    map: Map<memmap::Mmap>,
}

pub struct Builder<'a, 'kb> {
    commands: Vec<Exec>,
    binds: Vec<([u8; Layout::new::<Key>().size()], u64)>,
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
    pub fn bind<T: IntoExec>(&mut self, pattern: &str, cmd: T) {
        self.try_bind(pattern, cmd).expect("Unable to bind key");
    }
    pub fn try_bind<T: IntoExec>(&mut self, pattern: &str, cmd: T) -> Result<(), key::Error> {
        info!("mapping: {} -> {:?}", pattern, cmd);
        let key = Key::from_str(pattern)?;
        let cmd = cmd.into_exec()?;
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

    pub fn finish<T: AsRef<Path>>(mut self, path: T) -> io::Result<Controler> {
        info!("started building fst");
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
        info!("finished building fst");
        Ok(Controler { cmds, map })
    }
}

impl Controler {
    pub fn execute(&mut self, key: Key) {
        if let Some(index) = self
            .map
            .get::<[u8; Layout::new::<Key>().size()]>(key.into())
        {
            if usize::max_value()
                .try_into()
                .map(|v| index > v)
                .unwrap_or(true)
            {
                return;
            }
            let t = &mut self.cmds[index as usize];
            match t.spawn() {
                Ok(mut handle) => {
                    info!("spawned command | pid: {:?}", handle.id());
                    let _ = handle.try_wait(); // try to avoid zombies if possible
                }
                Err(err) => {
                    error!("unable to spawn command: {:?}", err);
                }
            }
        } else {
            warn!("unmatched combination {:?}", key);
        }
    }
}

fn fsterror_to_io(err: fst::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, err)
}
