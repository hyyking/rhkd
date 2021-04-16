use std::{alloc::Layout, ffi::CString, str::FromStr};

use crate::binds::xmodmap;

use x11::xlib::{self, XStringToKeysym};

pub type Error = ();

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C, packed)]
pub struct Key {
    pub sym: u64,
    pub mask: u32,
}
impl Key {
    pub const fn mask(mask: u32) -> Self {
        const SYM: u64 = u64::MAX;
        Self { sym: SYM, mask }
    }
    pub const fn sym(sym: u64) -> Self {
        Self { sym, mask: 0 }
    }
    pub const fn builder() -> Self {
        Self::mask(0)
    }

    pub const fn merge(mut self, other: Self) -> Self {
        if self.sym == u64::MAX {
            self.sym = other.sym
        }
        self.mask |= other.mask;
        self
    }
}

impl From<Key> for [u8; Layout::new::<Key>().size()] {
    fn from(key: Key) -> [u8; Layout::new::<Key>().size()] {
        unsafe { std::mem::transmute(key) }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Locks {
    pub num: Option<u32>,
    pub caps: Option<u32>,
}

impl Locks {
    pub fn new() -> Self {
        let num = match "Num_Lock" {
            xmodmap::MOD1 => Some(xlib::Mod1Mask),
            xmodmap::MOD2 => Some(xlib::Mod2Mask),
            xmodmap::MOD3 => Some(xlib::Mod3Mask),
            xmodmap::MOD4 => Some(xlib::Mod4Mask),
            xmodmap::MOD5 => Some(xlib::Mod5Mask),
            _ => None,
        };
        let caps = match "Caps_Lock" {
            xmodmap::MOD1 => Some(xlib::Mod1Mask),
            xmodmap::MOD2 => Some(xlib::Mod2Mask),
            xmodmap::MOD3 => Some(xlib::Mod3Mask),
            xmodmap::MOD4 => Some(xlib::Mod4Mask),
            xmodmap::MOD5 => Some(xlib::Mod5Mask),
            _ => None,
        };
        Self { num, caps }
    }
}

#[allow(unreachable_code)]
fn parse_convert_modifier(k: &str) -> Result<Key, String> {
    match k {
        "any" => Ok(Key::mask(xlib::AnyModifier)),
        "shift" => Ok(Key::mask(xlib::ShiftMask)),
        "ctrl" | "control" => Ok(Key::mask(xlib::ControlMask)),
        "lock" => Ok(Key::mask(xlib::LockMask)),
        "mod1" | xmodmap::MOD1 => Ok(Key::mask(xlib::Mod1Mask)),
        "mod2" | xmodmap::MOD2 => Ok(Key::mask(xlib::Mod2Mask)),
        "mod3" | xmodmap::MOD3 => Ok(Key::mask(xlib::Mod3Mask)),
        "mod4" | xmodmap::MOD4 => Ok(Key::mask(xlib::Mod4Mask)),
        "mod5" | xmodmap::MOD5 => Ok(Key::mask(xlib::Mod5Mask)),
        sym => into_keysym(sym).map(Key::sym),
    }
}

fn into_keysym(key: &str) -> Result<u64, String> {
    let cs = CString::new(key).expect("couldn't create new cstring");
    match unsafe { XStringToKeysym(cs.as_ptr()) } {
        0 => Err(format!("Unmatched key: {}", key)),
        a => Ok(a),
    }
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(input: &str) -> Result<Key, Self::Err> {
        let mut key = Key::builder();

        for k in input.split('+') {
            key = key.merge(parse_convert_modifier(k.trim()).map_err(|_| ())?);
        }
        Ok(key)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            parse_convert_modifier("ctrl").unwrap(),
            Key::mask(xlib::ControlMask)
        )
    }

    #[test]
    fn parse_mutliple() {
        let key = Key::from_str("ctrl + a").unwrap();
        assert_eq!(key.mask, xlib::ControlMask);
        assert_eq!(key.sym, into_keysym("a").unwrap());
    }
}
