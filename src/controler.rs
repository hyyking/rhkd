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
    binds: Vec<([u8; 12], u64)>,
    grab: GrabContext<'a>,
}

impl<'a> Builder<'a> {
    pub fn new(grab: GrabContext<'a>, cap: usize) -> Self {
        Self {
            commands: Vec::with_capacity(cap),
            binds: Vec::with_capacity(cap),
            grab,
        }
    }

    pub fn bind(&mut self, pattern: &str, cmd: Command) -> &mut Self {
        use std::str::FromStr;

        let key = Key::from_str(pattern).unwrap();
        self.grab.grab(key);

        let index = self.commands.len();
        self.commands.push(cmd);
        self.binds.push((key.into(), index as u64));
        self
    }

    pub fn finish<T: AsRef<std::path::Path>>(mut self, path: T) -> io::Result<Controler<'a>> {
        let _ = fs::remove_file(&path);
        let file = File::with_options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let map = self.build_map(file).expect("couldn't build map");
        let cmds = self.commands.into_boxed_slice();

        Ok(Controler {
            cmds,
            map,
            _grab: Some(self.grab),
        })
    }

    fn build_map(&mut self, file: File) -> io::Result<Map<memmap::Mmap>> {
        self.binds.as_mut_slice().sort_unstable_by_key(|(b, _)| *b);
        let mut map = MapBuilder::new(BufWriter::new(&file)).map_err(fsterror_to_io)?;

        map.extend_iter(self.binds.drain(..))
            .map_err(fsterror_to_io)?;
        map.finish().map_err(fsterror_to_io)?;

        Map::new(unsafe { memmap::Mmap::map(&file)? }).map_err(fsterror_to_io)
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
