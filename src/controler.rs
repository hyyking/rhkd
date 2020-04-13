use std::{
    fs::{self, File},
    io::{self, BufWriter},
    str::FromStr,
};

use crate::{
    key::{Cmd, Key},
    listener::GrabContext,
};

use fst::{self, Map, MapBuilder};

pub struct Controler<'a> {
    cmds: Box<[Cmd]>,
    map: Map<memmap::Mmap>,
    _grab: Option<GrabContext<'a>>, // ungrabs the keys on drop
}

pub struct Builder<'a> {
    commands: Vec<Cmd>,
    binds: Vec<([u8; 12], u64)>,
    grab: GrabContext<'a>,
}

impl<'a> Builder<'a> {
    pub fn new(grab: GrabContext<'a>) -> Self {
        Self {
            commands: Vec::new(),
            binds: Vec::new(),
            grab,
        }
    }
    pub fn bind(&mut self, pattern: &str, cmd: &str) -> &mut Self {
        let key = Key::from_str(pattern).unwrap();
        let cmd = Cmd::from_str(cmd).unwrap();

        self.grab.grab(key);
        self.commands.push(cmd);

        self.binds.push((
            Into::<[u8; 12]>::into(key),
            (self.commands.len() - 1) as u64,
        ));

        self
    }

    pub fn finish(mut self, path: &std::path::Path) -> io::Result<Controler<'a>> {
        self.commands.shrink_to_fit();
        self.binds.sort_unstable_by_key(|k| k.0);

        let cmds = self.commands.into_boxed_slice();

        let _ = fs::remove_file(&path);

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
            _grab: Some(self.grab),
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
            let _ = self.cmds[index as usize].0.spawn();
        }
    }
}

fn fsterror_to_io(err: fst::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, err)
}
