use std::{
    fs::{self, File},
    io::{self, BufWriter},
    process::Command,
};

use crate::{key::Key, listener::GrabContext};

use fst::{self, Map, MapBuilder};

pub struct Controler<'a> {
    cmds: Box<[Command]>,
    map: Map<memmap::Mmap>,
    _grab: Option<GrabContext<'a>>, // ungrabs the keys on drop
}

pub struct Builder<'a> {
    commands: Vec<Command>,
    map: MapBuilder<BufWriter<File>>,
    grab: GrabContext<'a>,
}

impl<'a> Builder<'a> {
    pub fn new<T: AsRef<std::path::Path>>(path: T, grab: GrabContext<'a>) -> io::Result<Self> {
        let _ = fs::remove_file(&path);
        let file = File::with_options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(Self {
            commands: Vec::with_capacity(128),
            map: MapBuilder::new(BufWriter::new(file)).map_err(fsterror_to_io)?,
            grab,
        })
    }

    pub fn bind(&mut self, pattern: &str, cmd: Command) -> &mut Self {
        use std::str::FromStr;

        let key = Key::from_str(pattern).unwrap();
        self.grab.grab(key);

        let index = self.commands.len();
        self.commands.push(cmd);
        self.map
            .insert(Into::<[u8; 12]>::into(key), index as u64)
            .expect("couldn't access map");
        self
    }

    pub fn finish(self) -> io::Result<Controler<'a>> {
        let cmds = self.commands.into_boxed_slice();
        let file = self
            .map
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
        if let Some(index) = self.map.get::<[u8; 12]>(dbg!(key.into())) {
            if index > usize::max_value().try_into().unwrap() {
                return;
            }
            let _ = self.cmds[index as usize].spawn();
        }
    }
}

fn fsterror_to_io(err: fst::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, err)
}
