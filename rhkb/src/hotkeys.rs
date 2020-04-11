use std::{cmp::Ordering, fs::File, io::BufWriter, process::Command};

use rhkb_lib::{
    keyboard::{tryinto_keysym, Key},
    Grabber,
};

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

    pub fn bind(&mut self, modifiers: &[u32], key: &str, cmd: Command) -> &mut Self {
        let index = self.commands.len();
        self.commands.push(cmd);
        let key = Key {
            mask: modifiers.into_iter().fold(0, |b, c| b | c),
            sym: tryinto_keysym(key).expect(&format!("invalid key {}", key)),
        };
        self.grab.grab(key);
        self.binds.push((key, index as u64));
        self
    }

    pub fn finish<T: AsRef<std::path::Path>>(mut self, path: T) -> std::io::Result<Controler<'a>> {
        let file = File::with_options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        self.binds
            .as_mut_slice()
            .sort_unstable_by_key(|(el, _)| *el);

        let grab = &self.grab;
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
            grab: Some(self.grab),
        })
    }
}

pub struct Controler<'a> {
    cmds: Box<[Command]>,
    map: Map<memmap::Mmap>,
    grab: Option<Grabber<'a>>,
}

impl<'a> Controler<'a> {
    pub fn execute(&mut self, key: Key) {
        if let Some(index) = self.map.get::<[u8; 12]>(dbg!(key.into())) {
            dbg!(self.cmds[index as usize].spawn());
        }
    }
}

/*
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
            Err(key) => {
                let _ = self.current.add_key(key);
            }
        }
    }
    pub fn register_release(&mut self, key: u16) {
        if let Ok(ctrl) = is_control_character(key) {
            self.current.remove_mod(ctrl);
        } else {
            let _ = self.current.pop();
        }
    }
}*/
