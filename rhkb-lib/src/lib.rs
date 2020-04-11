#![feature(try_blocks)]

pub mod fddriver;
pub mod keyboard;

use std::{cell::Cell, io, marker::PhantomData, task::Poll};

use fddriver::FdDriver;
use keyboard::Key;

use x11::xlib::{Display, XEvent, XKeyPressedEvent, XKeyReleasedEvent};

thread_local! {
    static GRABBED: Cell<bool> = Cell::new(false);
}

#[derive(Debug)]
pub enum Event {
    KeyPress(Key),
    KeyRelease(Key),
    Other,
}

pub struct KeyboardInputStream {
    display: *mut Display,
    root: u64,
    event_buff: XEvent,
    driver: FdDriver,
    pending: i32,
}

pub struct Grabber<'a> {
    display: *mut Display,
    window: u64,
    _a: PhantomData<&'a ()>,
}

impl<'a> Grabber<'a> {
    fn new(display: *mut Display, window: u64) -> io::Result<Self> {
        GRABBED.with(|grab| {
            if grab.get() {
                return Err(io::ErrorKind::AlreadyExists.into());
            }
            grab.set(true);
            Ok(Self {
                display,
                window,
                _a: PhantomData,
            })
        })
    }

    pub fn grab(&self, key: Key) {
        unsafe {
            x11::xlib::XGrabKey(
                self.display,
                self.keysym_to_keycode(key.sym),
                key.mask,
                self.window,
                1,
                1,
                1,
            );
        }
    }

    unsafe fn keysym_to_keycode(&self, sym: u64) -> i32 {
        x11::xlib::XKeysymToKeycode(self.display, sym) as i32
    }
}

impl<'a> Drop for Grabber<'a> {
    fn drop(&mut self) {
        use x11::xlib::{AnyKey, AnyModifier, XUngrabKey};
        GRABBED.with(|grab| {
            if !grab.get() {
                panic!("unexpected grab state")
            }
            unsafe { XUngrabKey(self.display, AnyKey, AnyModifier, self.window) };
            grab.set(false);
        })
    }
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

    pub fn grabber<'b>(&self) -> io::Result<Grabber<'b>> {
        Grabber::new(self.display, self.root)
    }

    pub fn poll(&mut self) -> Poll<Event> {
        use x11::xlib::XPending;

        self.pending = unsafe { XPending(self.display) };

        if self.pending == 0 {
            match self.driver.poll_read() {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => {
                    eprintln!("{:?}", e); // TODO: Handle
                    return Poll::Pending;
                }
                Poll::Ready(Ok(_)) => {}
            }
        }
        self.read_event();
        Poll::Ready(self.decode_event())
    }

    fn read_event(&mut self) {
        if self.pending > 0 {
            self.pending -= 1;
        }
        unsafe { x11::xlib::XNextEvent(self.display, &mut self.event_buff) };
    }

    unsafe fn keycode_to_keysym(&self, code: u8) -> u64 {
        x11::xlib::XKeycodeToKeysym(self.display, code, 0)
    }

    fn decode_event(&mut self) -> Event {
        use x11::xlib::{KeyPress as KEY_PRESS, KeyRelease as KEY_RELEASE};

        match self.event_buff.get_type() {
            KEY_PRESS => {
                let (sym, mask) = unsafe {
                    let event = XKeyPressedEvent::from(self.event_buff);
                    let sym = self.keycode_to_keysym(event.keycode as u8);
                    (sym, event.state)
                };
                Event::KeyPress(Key { sym, mask })
            }
            KEY_RELEASE => {
                let (sym, mask) = unsafe {
                    let event = XKeyReleasedEvent::from(self.event_buff);
                    let sym = self.keycode_to_keysym(event.keycode as u8);
                    (sym, event.state)
                };
                Event::KeyRelease(Key { sym, mask })
            }
            _ => Event::Other,
        }
    }
}
