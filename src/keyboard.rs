use std::{io, mem::MaybeUninit, os::unix::io::RawFd, ptr::NonNull};

use super::key::Key;

use mio::{event::Source, unix::SourceFd};

use x11::xlib::{
    AnyKey, AnyModifier, BadAccess as BAD_ACCESS, BadValue as BAD_VALUE, BadWindow as BAD_WINDOW,
    Display, KeyPress as KEY_PRESS, KeyRelease as KEY_RELEASE, Window, XCloseDisplay,
    XConnectionNumber, XDefaultScreenOfDisplay, XEvent, XGrabKey, XKeyPressedEvent,
    XKeyReleasedEvent, XKeycodeToKeysym, XKeysymToKeycode, XNextEvent, XOpenDisplay, XPending,
    XRootWindowOfScreen, XUngrabKey,
};

#[derive(Debug)]

pub struct DisplayContext {
    display: NonNull<Display>,
    root: Window,
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
    /// # Errors
    /// Will throw an error if the file can't be open
    pub fn current() -> io::Result<Self> {
        unsafe {
            let display = NonNull::new(XOpenDisplay(std::ptr::null())).ok_or({
                error!("unable to access X11 server");
                io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    "unable to access X11 server",
                )
            })?;
            let root = XRootWindowOfScreen(XDefaultScreenOfDisplay(display.as_ptr()));
            let fd = XConnectionNumber(display.as_ptr());

            trace!("connected to X11 server");
            Ok(Self { display, root, fd })
        }
    }

    pub fn display_mut(&mut self) -> &mut Display {
        unsafe { self.display.as_mut() }
    }
}

impl<'a> Keyboard<'a> {
    pub fn new(display: &'a mut DisplayContext) -> Self {
        let event = MaybeUninit::zeroed();
        Self { display, event }
    }

    pub fn grab_key(&mut self, key: Key) -> io::Result<()> {
        trace!("grabing {:?}", key);

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
        let res = match err as u8 {
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
        };
        if let Err(ref e) = res {
            error!("unable to grab {:?}, {}", key, e);
        }
        res
    }

    pub fn read_events(&mut self, buf: &mut Vec<Event>) {
        let in_flight = unsafe { XPending(self.display.display_mut()) };
        for _ in 0..in_flight {
            self.read_event();
            // SAFETY: We just read an event
            buf.push(unsafe { self.decode_event() });
        }
    }

    pub fn read_event(&mut self) {
        trace!("reading X11 event");
        unsafe { XNextEvent(self.display.display_mut(), self.event.as_mut_ptr()) };
    }

    pub unsafe fn decode_event(&mut self) -> Event {
        trace!("decoding X11 event");
        let event = &*self.event.as_ptr();
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
        unsafe { XKeycodeToKeysym(self.display.display_mut(), code as u8, 0) }
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
        unsafe { XCloseDisplay(self.display.as_ptr()) };
    }
}

impl<'a> Drop for Keyboard<'a> {
    fn drop(&mut self) {
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
