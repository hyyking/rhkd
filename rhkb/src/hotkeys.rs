use std::{
    fs::{remove_file, File},
    io::BufWriter,
    process::Command,
};

use rhkb_lib::{keyboard::Key, Grabber};

use fst::{Map, MapBuilder};

pub struct Builder<'a> {
    commands: Vec<Command>,
    binds: Vec<(Key, u64)>,
    grab: Grabber<'a>,
}

impl<'a> Builder<'a> {
    pub fn new(grab: Grabber<'a>, cap: usize) -> Self {
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
        self.binds.push((key, index as u64));
        self
    }

    pub fn finish<T: AsRef<std::path::Path>>(mut self, path: T) -> std::io::Result<Controler<'a>> {
        let _ = remove_file(&path);
        let file = File::with_options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        self.binds
            .as_mut_slice()
            .sort_unstable_by_key(|(el, _)| *el);

        let build = self
            .binds
            .into_iter()
            .map(|(key, map)| (dbg!(Into::<[u8; 12]>::into(key)), map));

        let mut map = MapBuilder::new(BufWriter::new(&file)).unwrap();

        map.extend_iter(build)
            .expect("can't bind the same pattern twice");

        map.finish().unwrap();

        let mm = unsafe { memmap::Mmap::map(&file)? };
        Ok(Controler {
            cmds: self.commands.into_boxed_slice(),
            map: Map::new(mm).expect("coudln't create map"),
            _grab: Some(self.grab),
        })
    }
}

pub struct Controler<'a> {
    cmds: Box<[Command]>,
    map: Map<memmap::Mmap>,
    _grab: Option<Grabber<'a>>, // ungrabs the keys on drop
}

impl<'a> Controler<'a> {
    pub fn execute(&mut self, key: Key) {
        if let Some(index) = self.map.get::<[u8; 12]>(dbg!(key.into())) {
            dbg!(self.cmds[index as usize].spawn());
        }
    }
}
