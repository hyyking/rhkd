pub const ESC: u16 = 1;
pub const CTRL: u16 = 29;
pub const ENTER: u16 = 28;
pub const L_SHIFT: u16 = 42;
pub const R_SHIFT: u16 = 53;
pub const ALT: u16 = 56;
pub const MOD: u16 = 125;
pub const SPACE: u16 = 57;
pub const TAB: u16 = 15;
pub const MAJ: u16 = 58;

pub const LEFT: u16 = 105;
pub const RIGHT: u16 = 106;
pub const UP: u16 = 103;
pub const DOWN: u16 = 108;

pub mod french {
    pub const A: u16 = 16;
    pub const Z: u16 = 17;
    pub const E: u16 = 18;
    pub const R: u16 = 19;
    pub const T: u16 = 20;
    pub const Y: u16 = 21;
    pub const U: u16 = 22;
    pub const I: u16 = 23;
    pub const O: u16 = 24;
    pub const P: u16 = 25;
    pub const Q: u16 = 30;
    pub const S: u16 = 31;
    pub const D: u16 = 32;
    pub const F: u16 = 33;
    pub const G: u16 = 34;
    pub const H: u16 = 35;
    pub const J: u16 = 36;
    pub const K: u16 = 37;
    pub const L: u16 = 38;
    pub const M: u16 = 39;
    pub const W: u16 = 44;
    pub const X: u16 = 45;
    pub const C: u16 = 46;
    pub const V: u16 = 47;
    pub const B: u16 = 48;
    pub const N: u16 = 49;
}

use std::convert::TryInto;
use std::ffi::CString;
use x11::xlib::XStringToKeysym;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct KeyCommand {
    pub mask: u32,
    pub sym: u64,
}

enum KeySym {
    Mask(u32),
    Code(u64),
}

pub fn into_keysym(key: &str) -> u64 {
    CString::new(key)
        .map(|cs| unsafe { XStringToKeysym(cs.as_ptr()) })
        .expect("ffi::NulError")
}

impl TryInto<KeySym> for &'static str {
    type Error = std::ffi::NulError;

    fn try_into(self) -> Result<KeySym, Self::Error> {
        let keysim = CString::new(self).map(|cs| unsafe { XStringToKeysym(cs.as_ptr()) })?;
        Ok(KeySym::Code(keysim as u64))
    }
}

impl TryInto<KeySym> for &u32 {
    type Error = std::ffi::NulError;
    fn try_into(self) -> Result<KeySym, Self::Error> {
        Ok(KeySym::Mask(*self))
    }
}
