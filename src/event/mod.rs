mod fd;
pub mod signal;

use std::{cell::Cell, io, marker::PhantomData, ptr::NonNull, task::Poll};

use self::fd::Driver;
use super::key::Key;

use x11::xlib::{Display, Window, XEvent};

thread_local! {
    static GRABBED: Cell<bool> = Cell::new(false);
}

#[derive(Debug)]
pub enum Event {
    KeyPress(Key),
    KeyRelease(Key),
    Other,
}

pub struct Keyboard {
    display: NonNull<Display>,
    root: Window,
    event_buff: XEvent,
    driver: Driver,
}

pub struct GrabContext<'a> {
    display: NonNull<Display>,
    window: u64,
    _m: PhantomData<&'a Display>,
}

impl<'a> GrabContext<'a> {
    fn new(display: NonNull<Display>, window: u64) -> io::Result<Self> {
        GRABBED.with(|grab| {
            if grab.get() {
                return Err(io::ErrorKind::AlreadyExists.into());
            }
            grab.set(true);
            Ok(Self {
                display,
                window,
                _m: PhantomData,
            })
        })
    }

    pub fn grab_key(&mut self, key: Key) -> io::Result<()> {
        use x11::xlib::{BadAccess, BadValue, BadWindow, XGrabKey, XKeysymToKeycode};
        const BAD_ACCESS: i32 = BadAccess as i32;
        const BAD_VALUE: i32 = BadValue as i32;
        const BAD_WINDOW: i32 = BadWindow as i32;
        let err = unsafe {
            let code = XKeysymToKeycode(self.display.as_ptr(), key.sym);
            XGrabKey(
                self.display.as_ptr(),
                i32::from(code),
                key.mask,
                self.window,
                i32::from(true),
                x11::xlib::GrabModeSync,
                x11::xlib::GrabModeAsync,
            )
        };
        match err {
            BAD_ACCESS => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("X11 BadAccess: {:?}", key),
            )),
            BAD_VALUE => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("X11 BadValue: {:?}", key),
            )),
            BAD_WINDOW => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("X11 BadWindow: {:?}", key),
            )),
            _ => Ok(()),
        }
    }
}

impl Keyboard {
    /// # Errors
    /// Will throw an error if the file can't be open
    pub fn new() -> io::Result<Self> {
        use x11::xlib::{
            XConnectionNumber, XDefaultScreenOfDisplay, XOpenDisplay, XRootWindowOfScreen,
        };

        let (display, root, driver) = unsafe {
            let display = NonNull::new(XOpenDisplay(std::ptr::null())).ok_or({
                io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    "unable to access x11 server",
                )
            })?;

            let root = XRootWindowOfScreen(XDefaultScreenOfDisplay(display.as_ptr()));
            let driver = Driver::new(XConnectionNumber(display.as_ptr()));

            (display, root, driver)
        };

        Ok(Self {
            display,
            root,
            driver,
            event_buff: XEvent { pad: [0; 24] },
        })
    }

    pub fn context<'b>(&mut self) -> io::Result<GrabContext<'b>> {
        GrabContext::new(self.display, self.root)
    }

    pub fn poll(&mut self) -> Poll<Event> {
        use x11::xlib::XPending;

        if unsafe { XPending(self.display.as_ptr()) } == 0 {
            match self.driver.poll_read() {
                Poll::Ready(Ok(_)) => {}
                _ => return Poll::Pending,
            }
        }
        self.read_event();
        Poll::Ready(self.decode_event())
    }

    fn read_event(&mut self) {
        unsafe { x11::xlib::XNextEvent(self.display.as_ptr(), &mut self.event_buff) };
    }
    fn decode_event(&mut self) -> Event {
        use x11::xlib::{
            KeyPress as KEY_PRESS, KeyRelease as KEY_RELEASE, XKeyPressedEvent, XKeyReleasedEvent,
        };
        match self.event_buff.get_type() {
            KEY_PRESS => {
                let (sym, mask) = unsafe {
                    let event = XKeyPressedEvent::from(self.event_buff);
                    let sym = self.keycode_to_keysym(event.keycode);
                    (sym, event.state)
                };
                Event::KeyPress(Key { sym, mask })
            }
            KEY_RELEASE => {
                let (sym, mask) = unsafe {
                    let event = XKeyReleasedEvent::from(self.event_buff);
                    let sym = self.keycode_to_keysym(event.keycode);
                    (sym, event.state)
                };
                Event::KeyRelease(Key { sym, mask })
            }
            _ => Event::Other,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    unsafe fn keycode_to_keysym(&mut self, code: u32) -> u64 {
        x11::xlib::XKeycodeToKeysym(self.display.as_ptr(), code as u8, 0)
    }
}

impl<'a> Drop for GrabContext<'a> {
    fn drop(&mut self) {
        use x11::xlib::{AnyKey, AnyModifier, XUngrabKey};
        GRABBED.with(|grab| {
            (!grab.get()).then(|| panic!("unexpected GrabContext state"));

            unsafe { XUngrabKey(self.display.as_ptr(), AnyKey, AnyModifier, self.window) };
            grab.set(false);
        })
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        use x11::xlib::XCloseDisplay;
        assert!(!GRABBED.with(Cell::get));
        unsafe { XCloseDisplay(self.display.as_ptr()) };
    }
}
