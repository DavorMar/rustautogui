

#[cfg(target_os = "linux")]
use crate::mouse::platform::Mouse;
#[cfg(target_os = "linux")]
use x11::xlib::*;
#[cfg(target_os = "linux")]
use std::ptr;

#[cfg(any(target_os = "windows", target_os = "macos"))]
use crate::mouse::platform::Mouse;

use std::time::Duration;
use std::thread::sleep;

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
#[cfg(target_os = "linux")]
pub fn print_mouse_position() {
    let display_wrapper = DisplayWrapper::new();

    unsafe {
        let screen = XDefaultScreen(display_wrapper.display);
        let root = XRootWindow(display_wrapper.display, screen);
        let mouse = Mouse::new(Some(display_wrapper.display), Some(root));
        loop {
            let (x, y) = mouse.get_mouse_position();
            println!("{x}, {y}");
            sleep(Duration::from_millis(20));
        }
    } 
}





#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn print_mouse_position() {
    loop {
        let (x,y) = Mouse::get_mouse_position();
        println!("{x}, {y}");
        sleep(Duration::from_millis(20));
    };
}
