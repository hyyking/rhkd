pub mod keyboard;

use std::{
    fs::File,
    io::{self, Read},
    mem,
    path::Path,
};

use keyboard::KeyCommand;
use x11::xlib::{
    self, Display, KeyPress as KEY_PRESS, KeyRelease as KEY_RELEASE, XDefaultScreenOfDisplay,
    XEvent, XFlush, XKeyPressedEvent, XKeyReleasedEvent, XKeycodeToKeysym, XNextEvent,
    XOpenDisplay, XRootWindowOfScreen,
};

#[derive(Debug)]
pub enum Key {
    Press(KeyCommand),
    Release(KeyCommand),
    Other,
}

pub struct KeyboardInputStream {
    display: *mut Display,
    root: u64,
    event_buff: XEvent,
}

impl KeyboardInputStream {
    /// # Errors
    /// Will throw an error if the file can't be open
    pub fn new() -> io::Result<Self> {
        let event_buff = XEvent { pad: [0; 24] };
        let (display, root) = unsafe {
            let display = XOpenDisplay(std::ptr::null());

            if display == std::ptr::null_mut() {
                panic!("Couldn't access display {}", env!("DISPLAY"));
            }

            let screen = XDefaultScreenOfDisplay(display);
            let root = XRootWindowOfScreen(screen);
            (display, root)
        };

        Ok(Self {
            display,
            root,
            event_buff,
        })
    }
    pub fn grab<'a, T>(&self, grab: T) -> io::Result<()>
    where
        T: IntoIterator<Item = &'a KeyCommand>,
    {
        for key in grab.into_iter() {
            unsafe {
                xlib::XGrabKey(
                    self.display,
                    xlib::XKeysymToKeycode(self.display, key.sym) as i32,
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
}

impl Iterator for KeyboardInputStream {
    type Item = io::Result<Key>;

    fn next(&mut self) -> Option<Self::Item> {
        dbg!(self.event_buff);
        unsafe {
            XNextEvent(self.display, &mut self.event_buff);
        }

        let event_type = self.event_buff.get_type();
        let event = match event_type {
            KEY_PRESS => unsafe {
                let event = XKeyPressedEvent::from(self.event_buff);
                let key = KeyCommand {
                    sym: XKeycodeToKeysym(self.display, event.keycode as u8, 0) as u64,
                    mask: 0xEF & event.state as u32,
                };
                Key::Press(key)
            },
            KEY_RELEASE => unsafe {
                let event = XKeyReleasedEvent::from(self.event_buff);
                let key = KeyCommand {
                    sym: XKeycodeToKeysym(self.display, event.keycode as u8, 0) as u64,
                    mask: 0xEF & event.state as u32,
                };
                Key::Release(key)
            },
            _ => Key::Other,
        };
        Some(Ok(event))
    }
}
