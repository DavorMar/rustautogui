use crate::errors::AutoGuiError;
#[cfg(target_os = "linux")]
use crate::mouse::platform::Mouse;
#[cfg(target_os = "linux")]
use std::ptr;
#[cfg(target_os = "linux")]
use x11::xlib::*;

#[cfg(any(target_os = "windows", target_os = "macos"))]
use crate::core::mouse::platform::Mouse;

use std::thread::sleep;
use std::time::Duration;

/*

small helper function to open a window that shows mouse position



example :
fn main() {
    mouse::mouse_position::show_mouse_position_window();
}
    thats all
*/
#[cfg(target_os = "linux")]
struct DisplayWrapper {
    display: *mut x11::xlib::Display,
}
//created so display gets dropped when code finishes
#[cfg(target_os = "linux")]
impl DisplayWrapper {
    fn new() -> Self {
        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                panic!("Unable to open X display");
            }
            DisplayWrapper { display }
        }
    }
}
#[cfg(target_os = "linux")]
impl Drop for DisplayWrapper {
    fn drop(&mut self) {
        unsafe {
            XCloseDisplay(self.display);
        }
    }
}

pub fn print_mouse_position() -> Result<(), AutoGuiError> {
    #[cfg(target_os = "linux")]
    {
        let display_wrapper = DisplayWrapper::new();

        unsafe {
            let screen = XDefaultScreen(display_wrapper.display);
            let root = XRootWindow(display_wrapper.display, screen);
            let mouse = Mouse::new(display_wrapper.display, root);
            loop {
                let (x, y) = mouse.get_mouse_position()?;
                println!("{x}, {y}");
                sleep(Duration::from_millis(20));
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        loop {
            let (x, y) = Mouse::get_mouse_position();
            println!("{x}, {y}");
            sleep(Duration::from_millis(20));
        }
    }
    #[cfg(target_os = "macos")]
    {
        loop {
            let (x, y) = Mouse::get_mouse_position()?;
            println!("{x}, {y}");
            sleep(Duration::from_millis(20));
        }
    }
}
