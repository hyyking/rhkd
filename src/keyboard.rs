use std::{io, mem::MaybeUninit, os::unix::io::RawFd, ptr::NonNull};

use super::key::Key;

use mio::{event::Source, unix::SourceFd};
use x11::xlib::{Display, Window, XEvent};

#[derive(Debug)]

pub struct DisplayContext {
    root: Window,
    display: NonNull<Display>,
    fd: RawFd,
}

pub struct Keyboard<'a> {
    display: &'a mut DisplayContext,
    event: MaybeUninit<XEvent>,
}

pub enum Event {
    KeyPress(Key),
    KeyRelease(Key),
    Other,
}

impl DisplayContext {
    pub fn current() -> io::Result<Self> {
        use x11::xlib::{
            XConnectionNumber, XDefaultScreenOfDisplay, XOpenDisplay, XRootWindowOfScreen,
        };
        unsafe {
            let display = NonNull::new(XOpenDisplay(std::ptr::null())).ok_or({
                io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    "unable to access x11 server",
                )
            })?;
            let root = XRootWindowOfScreen(XDefaultScreenOfDisplay(display.as_ptr()));
            let fd = XConnectionNumber(display.as_ptr());
            Ok(Self { display, root, fd })
        }
    }

    pub fn display_mut(&mut self) -> &mut Display {
        unsafe { self.display.as_mut() }
    }
}

impl<'a> Keyboard<'a> {
    /// # Errors
    /// Will throw an error if the file can't be open
    pub fn new(display: &'a mut DisplayContext) -> Self {
        let event = MaybeUninit::zeroed();
        Self { display, event }
    }

    pub fn grab_key(&mut self, key: Key) -> io::Result<()> {
        use x11::xlib::{BadAccess, BadValue, BadWindow, XGrabKey, XKeysymToKeycode};
        const BAD_ACCESS: i32 = BadAccess as i32;
        const BAD_VALUE: i32 = BadValue as i32;
        const BAD_WINDOW: i32 = BadWindow as i32;
        let err = unsafe {
            let code = XKeysymToKeycode(self.display.display_mut(), key.sym);
            XGrabKey(
                self.display.display_mut(),
                i32::from(code),
                key.mask,
                self.display.root,
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

    pub fn read_event(&mut self) {
        unsafe { x11::xlib::XNextEvent(self.display.display_mut(), self.event.as_mut_ptr()) };
    }

    pub fn decode_event(&mut self) -> Event {
        use x11::xlib::{
            KeyPress as KEY_PRESS, KeyRelease as KEY_RELEASE, XKeyPressedEvent, XKeyReleasedEvent,
        };
        let event = unsafe { &*self.event.as_mut_ptr() };
        match event.get_type() {
            KEY_PRESS => {
                let (sym, mask) = {
                    let event = XKeyPressedEvent::from(event);
                    let sym = self.keycode_to_keysym(event.keycode);
                    (sym, event.state)
                };
                Event::KeyPress(Key { sym, mask })
            }
            KEY_RELEASE => {
                let (sym, mask) = {
                    let event = XKeyReleasedEvent::from(event);
                    let sym = self.keycode_to_keysym(event.keycode);
                    (sym, event.state)
                };
                Event::KeyRelease(Key { sym, mask })
            }
            _ => Event::Other,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn keycode_to_keysym(&mut self, code: u32) -> u64 {
        unsafe { x11::xlib::XKeycodeToKeysym(self.display.display_mut(), code as u8, 0) }
    }
}

impl<'a> Source for Keyboard<'a> {
    fn register(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interest,
    ) -> io::Result<()> {
        SourceFd(&self.display.fd).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        interests: mio::Interest,
    ) -> io::Result<()> {
        SourceFd(&self.display.fd).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &mio::Registry) -> io::Result<()> {
        SourceFd(&self.display.fd).deregister(registry)
    }
}

impl Drop for DisplayContext {
    fn drop(&mut self) {
        use x11::xlib::XCloseDisplay;
        unsafe { XCloseDisplay(self.display.as_ptr()) };
    }
}

impl<'a> Drop for Keyboard<'a> {
    fn drop(&mut self) {
        use x11::xlib::{AnyKey, AnyModifier, XUngrabKey};
        unsafe {
            XUngrabKey(
                self.display.display_mut(),
                AnyKey,
                AnyModifier,
                self.display.root,
            )
        };
    }
}
