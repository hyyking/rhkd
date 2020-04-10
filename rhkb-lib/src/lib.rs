pub mod fddriver;
pub mod keyboard;

use std::{io, task::Poll};

use keyboard::Key;
use x11::xlib::{Display, XEvent, XKeyPressedEvent, XKeyReleasedEvent};

#[derive(Debug)]
pub enum Event {
    KeyPress(Key),
    KeyRelease(Key),
    Other,
}

use fddriver::FdDriver;

pub struct KeyboardInputStream {
    display: *mut Display,
    root: u64,
    event_buff: XEvent,
    driver: FdDriver,
    pending: i32,
}

impl KeyboardInputStream {
    /// # Errors
    /// Will throw an error if the file can't be open
    pub fn new() -> io::Result<Self> {
        use x11::xlib::{
            XConnectionNumber, XDefaultScreenOfDisplay, XOpenDisplay, XRootWindowOfScreen,
        };

        let (display, root) = unsafe {
            let display = XOpenDisplay(std::ptr::null());
            if display == std::ptr::null_mut() {
                return Err(io::ErrorKind::AddrNotAvailable.into());
            }

            let screen = XDefaultScreenOfDisplay(display);
            let root = XRootWindowOfScreen(screen);

            (display, root)
        };
        let driver = FdDriver::new(unsafe { XConnectionNumber(display) });
        Ok(Self {
            display,
            root,
            driver,
            event_buff: XEvent { pad: [0; 24] },
            pending: 0,
        })
    }
    pub fn grab<'a, T>(&self, grab: T) -> io::Result<()>
    where
        T: IntoIterator<Item = &'a Key>,
    {
        use x11::xlib::XGrabKey;

        for key in grab.into_iter() {
            unsafe {
                XGrabKey(
                    self.display,
                    self.keysym_to_keycode(key.sym),
                    key.mask,
                    self.root,
                    1,
                    1,
                    1,
                );
            }
        }
        Ok(())
    }

    pub fn poll(&mut self) -> Poll<Event> {
        use x11::xlib::XPending;

        self.pending = unsafe { XPending(self.display) };

        if self.pending == 0 {
            match self.driver.poll_read() {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => {
                    eprintln!("{:?}", e);
                    return Poll::Pending;
                }
                Poll::Ready(Ok(_)) => {}
            }
        }
        self.read_event();
        Poll::Ready(self.decode_event())
    }

    unsafe fn keycode_to_keysym(&self, code: u8) -> u64 {
        x11::xlib::XKeycodeToKeysym(self.display, code, 0)
    }
    unsafe fn keysym_to_keycode(&self, sym: u64) -> i32 {
        x11::xlib::XKeysymToKeycode(self.display, sym) as i32
    }

    fn read_event(&mut self) {
        if self.pending > 0 {
            self.pending -= 1;
        }
        unsafe { x11::xlib::XNextEvent(self.display, &mut self.event_buff) };
    }

    fn decode_event(&mut self) -> Event {
        use x11::xlib::{KeyPress as KEY_PRESS, KeyRelease as KEY_RELEASE};

        match self.event_buff.get_type() {
            KEY_PRESS => {
                let (sym, mask) = unsafe {
                    let event = XKeyPressedEvent::from(self.event_buff);
                    let sym = self.keycode_to_keysym(event.keycode as u8);
                    // (sym, 0xEF & event.state)
                    (sym, event.state)
                };
                Event::KeyPress(Key { sym, mask })
            }
            KEY_RELEASE => {
                let (sym, mask) = unsafe {
                    let event = XKeyReleasedEvent::from(self.event_buff);
                    let sym = self.keycode_to_keysym(event.keycode as u8);
                    // (sym, 0xEF & event.state)
                    (sym, event.state)
                };
                Event::KeyRelease(Key { sym, mask })
            }
            _ => Event::Other,
        }
    }
}
