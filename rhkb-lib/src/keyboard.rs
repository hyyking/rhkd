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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Key {
    pub mask: u32,
    pub sym: u64,
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
pub fn tryinto_keysym(key: &str) -> Result<u64, std::ffi::NulError> {
    std::ffi::CString::new(key).map(|cs| unsafe { x11::xlib::XStringToKeysym(cs.as_ptr()) })
}
