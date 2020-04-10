use std::ffi::{CString, NulError};
use x11::xlib::XStringToKeysym;

pub mod mask {
    use x11::xlib;
    pub const ANY: u32 = xlib::AnyModifier;
    pub const MOD1: u32 = xlib::Mod1Mask;
    pub const MOD2: u32 = xlib::Mod2Mask;
    pub const MOD3: u32 = xlib::Mod3Mask;
    pub const MOD4: u32 = xlib::Mod4Mask;
    pub const MOD5: u32 = xlib::Mod5Mask;
    pub const SHIFT: u32 = xlib::ShiftMask;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Key {
    pub mask: u32,
    pub sym: u64,
}

pub fn tryinto_keysym(key: &str) -> Result<u64, NulError> {
    CString::new(key).map(|cs| unsafe { XStringToKeysym(cs.as_ptr()) })
}
