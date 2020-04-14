use libc::{c_void, sigaction as SigAction, sigaction, siginfo_t as SigInfo, SA_SIGINFO};
use std::mem::MaybeUninit;

pub struct SigHandler {
    inner: SigAction,
}

type Handler = fn(i32, SigInfo, *mut c_void);

impl SigHandler {
    pub fn new(f: Handler) -> Self {
        let mut sa: SigAction = unsafe { MaybeUninit::zeroed().assume_init() };
        sa.sa_flags = SA_SIGINFO;
        sa.sa_sigaction = f as usize;

        Self { inner: sa }
    }
    pub fn register(&mut self, code: i32) -> std::io::Result<()> {
        let c = unsafe { sigaction(code, &mut self.inner, std::ptr::null_mut()) };
        if c != 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
