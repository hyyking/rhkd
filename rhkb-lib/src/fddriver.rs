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
        let set = unsafe {
            let mut set: MaybeUninit<FdSet> = MaybeUninit::uninit();
            libc::FD_ZERO(set.as_mut_ptr());
            set.assume_init()
        };
        let timer = Timeval {
            tv_sec: 1,
            tv_usec: 0,
        };
        Self { fd, set, timer }
    }
    pub fn poll_read(&mut self) -> Poll<Result<(), i32>> {
        self.reset(); // rest state

        let nmb = unsafe {
            libc::select(
                self.fd + 1,
                &mut self.set,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut self.timer,
            )
        };
        match nmb {
            a if a < 0 => Poll::Ready(Err(a)),
            a if a == 0 => Poll::Pending,
            a if a > 0 => Poll::Ready(Ok(())),
            _ => unreachable!(),
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
