#[cfg(target_os = "linux")]
use super::Mouse;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use crate::core::mouse::Mouse;
use crate::errors::AutoGuiError;
use std::env;
#[cfg(target_os = "linux")]
use std::ptr;
#[cfg(target_os = "linux")]
use x11::xlib::*;

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
        let session = env::var("XDG_SESSION_TYPE").map_err(|_| {
            AutoGuiError::OSFailure(
                "XDG_SESSION_TYPE is not set, cannot determine wayland vs x11".to_string(),
            )
        })?;
        match session.as_str() {
            "wayland" => todo!(),
            "x11" => {
                let display_wrapper = DisplayWrapper::new();

                unsafe {
                    use crate::core::mouse::linux::X11Mouse;

                    let screen = XDefaultScreen(display_wrapper.display);
                    let root = XRootWindow(display_wrapper.display, screen);
                    let mouse = X11Mouse::new(display_wrapper.display, root);
                    loop {
                        let (x, y) = mouse.get_mouse_position()?;
                        println!("{x}, {y}");
                        sleep(Duration::from_millis(20));
                    }
                }
            }
            unknown => Err(AutoGuiError::OSFailure(format!(
                "Unknown XDG_SESSION_TYPE={unknown} - should be 'x11' or 'wayland'"
            ))),
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
