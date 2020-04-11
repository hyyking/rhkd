use std::io;
use std::mem::MaybeUninit;
use std::os::unix::io::RawFd;
use std::task::Poll;

use libc::{fd_set as FdSet, timeval as Timeval};

pub struct FdDriver {
    fd: RawFd,
    set: FdSet,
    timer: Timeval,
}

impl FdDriver {
    pub fn new(fd: RawFd) -> Self {
        // safety used by reset which is called before any read
        let set = unsafe { MaybeUninit::zeroed().assume_init() };

        let timer = Timeval {
            tv_sec: 1,
            tv_usec: 0,
        };
        Self { fd, set, timer }
    }
    pub fn poll_read(&mut self) -> Poll<io::Result<()>> {
        self.reset(); // reset state

        let sel = select(
            self.fd + 1,
            Some(&mut self.set),
            None,
            None,
            Some(&mut self.timer),
        );

        match sel.as_ref().map_err(|e| e.kind()) {
            Ok(_) => Poll::Ready(Ok(())),
            Err(io::ErrorKind::WouldBlock) => Poll::Pending,
            Err(_) => Poll::Ready(sel.map(|_| ())),
        }
    }

    fn reset(&mut self) {
        use libc::{FD_SET, FD_ZERO};

        self.timer = Timeval {
            tv_sec: 1,
            tv_usec: 0,
        };

        unsafe {
            FD_ZERO(&mut self.set);
            FD_SET(self.fd, &mut self.set);
        };
    }
}

// safe rust wrapper
fn select(
    fd: RawFd,
    rs: Option<&mut FdSet>,
    ws: Option<&mut FdSet>,
    es: Option<&mut FdSet>,
    tv: Option<&mut Timeval>,
) -> io::Result<i32> {
    let rs = rs.map(|r| r as *mut _).unwrap_or(std::ptr::null_mut());
    let ws = ws.map(|r| r as *mut _).unwrap_or(std::ptr::null_mut());
    let es = es.map(|r| r as *mut _).unwrap_or(std::ptr::null_mut());
    let tv = tv.map(|r| r as *mut _).unwrap_or(std::ptr::null_mut());

    let ret = unsafe { libc::select(fd, rs, ws, es, tv) };

    if ret <= 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}
