use std::{
    process::{Command, Stdio},
    str::FromStr,
};

use super::binds::xmodmap;

pub type Error = ();

#[repr(transparent)]
pub struct Cmd(pub Command);

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub mask: u32,
    pub sym: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Either<A, B> {
    A(A),
    B(B),
}

impl Into<[u8; 12]> for Key {
    fn into(self) -> [u8; 12] {
        let mask = self.mask.to_ne_bytes();
        let sym = self.sym.to_ne_bytes();
        [
            mask[0], mask[1], mask[2], mask[3], sym[0], sym[1], sym[2], sym[3], sym[4], sym[5],
            sym[6], sym[7],
        ]
    }
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(input: &str) -> Result<Key, Self::Err> {
        let mut key = Key { mask: 0, sym: 0 };

        for k in input.split('+') {
            match parse_convert_modifier(k.trim()) {
                Either::A(modifier) => key.mask |= modifier,
                Either::B(sym) => key.sym |= sym,
            }
        }
        Ok(key)
    }
}

impl FromStr for Cmd {
    type Err = Error;

    fn from_str(cmd: &str) -> Result<Self, Self::Err> {
        let mut bld: Option<Command> = None;

        for arg in cmd.split(' ') {
            if let Some(b) = bld.as_mut() {
                b.arg(arg);
            } else {
                bld.is_none().then(|| {
                    bld = Some(Command::new(arg));
                });
            }
        }
        let mut bld = bld.ok_or(())?;
        bld.stdin(Stdio::null());
        bld.stderr(Stdio::null());
        bld.stdout(Stdio::null());
        Ok(Self(bld))
    }
}

#[allow(unreachable_code)]
fn parse_convert_modifier(k: &str) -> Either<u32, u64> {
    use x11::xlib;
    match k {
        "any" => Either::A(xlib::AnyModifier),
        "shift" => Either::A(xlib::ShiftMask),
        "ctrl" | "control" => Either::A(xlib::ControlMask),
        "lock" => Either::A(xlib::LockMask),
        "mod1" | xmodmap::MOD1 => Either::A(xlib::Mod1Mask),
        "mod2" | xmodmap::MOD2 => Either::A(xlib::Mod2Mask),
        "mod3" | xmodmap::MOD3 => Either::A(xlib::Mod3Mask),
        "mod4" | xmodmap::MOD4 => Either::A(xlib::Mod4Mask),
        "mod5" | xmodmap::MOD5 => Either::A(xlib::Mod5Mask),
        k => Either::B(into_keysym(k).unwrap()),
    }
}

fn into_keysym(key: &str) -> Result<u64, String> {
    let cs = std::ffi::CString::new(key).expect("couldn't create new cstring");
    match unsafe { x11::xlib::XStringToKeysym(cs.as_ptr()) } {
        0 => Err(format!("Unmatched key: {}", key)),
        a => Ok(a),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use x11::xlib;

    #[test]
    fn parse() {
        assert_eq!(parse_convert_modifier("ctrl"), Either::A(xlib::ControlMask))
    }

    #[test]
    fn parse_mutliple() {
        let key = Key::from_str("ctrl + a").unwrap();
        assert_eq!(key.mask, xlib::ControlMask);
        assert_eq!(key.sym, into_keysym("a").unwrap());
    }
}
