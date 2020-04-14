mod fd;
pub mod signal;

use std::{cell::Cell, io, task::Poll};

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
    display: Option<Box<Display>>,
    root: Window,
    event_buff: XEvent,
    driver: Driver,
}

pub struct GrabContext<'a> {
    display: &'a mut Display,
    window: u64,
}

impl<'a> GrabContext<'a> {
    fn new(display: *mut Display, window: u64) -> io::Result<Self> {
        GRABBED.with(|grab| {
            if grab.get() {
                return Err(io::ErrorKind::AlreadyExists.into());
            }
            grab.set(true);
            Ok(Self {
                display: unsafe { &mut *display },
                window,
            })
        })
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn grab(&mut self, key: Key) -> io::Result<()> {
        use x11::xlib::{BadAccess as BAD_ACCESS, BadValue as BAD_VALUE, BadWindow as BAD_WINDOW};
        let code = unsafe {
            x11::xlib::XGrabKey(
                self.display,
                i32::from(self.keysym_to_keycode(key.sym)),
                key.mask,
                self.window,
                i32::from(true),
                x11::xlib::GrabModeSync,
                x11::xlib::GrabModeAsync,
            )
        };
        match code as u8 {
            BAD_ACCESS => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("X11 BadAccess {:?}", key),
            )),
            BAD_VALUE => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("X11 BadValue {:?}", key),
            )),
            BAD_WINDOW => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("X11 BadWindow {:?}", key),
            )),
            _ => Ok(()),
        }
    }

    unsafe fn keysym_to_keycode(&mut self, sym: u64) -> u8 {
        x11::xlib::XKeysymToKeycode(self.display, sym)
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
            let display = XOpenDisplay(std::ptr::null());

            display.is_null().then(|| ()).ok_or({
                io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    "unable to access x11 server",
                )
            })?;

            let root = XRootWindowOfScreen(XDefaultScreenOfDisplay(display));
            let driver = Driver::new(XConnectionNumber(display));

            (Box::from_raw(display), root, driver)
        };

        Ok(Self {
            display: Some(display),
            root,
            driver,
            event_buff: XEvent { pad: [0; 24] },
        })
    }
    fn get_display(&mut self) -> &mut Display {
        self.display.as_mut().expect("get_display after drop") // always available during execution
    }

    pub fn context<'b>(&mut self) -> io::Result<GrabContext<'b>> {
        GrabContext::new(self.get_display() as *mut _, self.root)
    }

    pub fn poll(&mut self) -> Poll<Event> {
        use x11::xlib::XPending;

        if unsafe { XPending(self.get_display()) } == 0 {
            match self.driver.poll_read() {
                Poll::Ready(Ok(_)) => {}
                Poll::Pending | Poll::Ready(Err(_)) => return Poll::Pending,
            }
        }
        self.read_event();
        Poll::Ready(self.decode_event())
    }

    fn read_event(&mut self) {
        unsafe { x11::xlib::XNextEvent(self.get_display(), &mut self.event_buff) };
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
        x11::xlib::XKeycodeToKeysym(self.get_display(), code as u8, 0)
    }
}

impl<'a> Drop for GrabContext<'a> {
    fn drop(&mut self) {
        use x11::xlib::{AnyKey, AnyModifier, XUngrabKey};
        GRABBED.with(|grab| {
            (!grab.get()).then(|| panic!("unexpected GrabContext state"));

            unsafe { XUngrabKey(self.display, AnyKey, AnyModifier, self.window) };
            grab.set(false);
        })
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        use x11::xlib::XCloseDisplay;
        assert!(!GRABBED.with(Cell::get));
        unsafe { XCloseDisplay(Box::into_raw(self.display.take().unwrap())) };
    }
}
