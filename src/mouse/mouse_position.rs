
use eframe::egui::{CentralPanel, Context, Pos2};
use eframe::{App, NativeOptions};

#[cfg(target_os = "linux")]
use crate::mouse::linux::Mouse;
#[cfg(target_os = "linux")]
use x11::xlib::*;
#[cfg(target_os = "linux")]
use std::ptr;

#[cfg(target_os = "windows")]
use crate::mouse::windows::Mouse;

/* 

small helper function to open a window that shows mouse position 



example :
fn main() {
    mouse::mouse_position::show_mouse_position_window();
}
    thats all
*/
#[cfg(target_os = "linux")]
fn get_mouse_position() -> Pos2 {
    unsafe {
        let display: *mut _XDisplay = XOpenDisplay(ptr::null());
        if display.is_null() {
            panic!("Unable to open X display");
        }

        // Get the root window
        let screen = XDefaultScreen(display);
        let root = XRootWindow(display, screen);
        let mouse = Mouse::new(Some(display), Some(root));
        let (x,y) = mouse.get_mouse_position();
        XCloseDisplay(display);
        Pos2 { x: x as f32, y: y as f32 }
        
    }
       
}


// #[cfg(target_os = "windows")]
// fn get_mouse_position() -> Pos2 {
//     let (x,y) = Mouse::get_mouse_position();
//     Pos2 { x: x as f32, y: y as f32 }
// }

// pub fn show_mouse_position_window() -> Result<(), eframe::Error> {
//     eframe::run_native(
//         "Mouse Position",
//         NativeOptions::default(),
//         Box::new(|_cc| Ok(Box::new(MyApp))),
//     )
// }

// struct MyApp;

// impl App for MyApp {
//     fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
//         let mouse_pos = get_mouse_position();
        
//         CentralPanel::default().show(ctx, |ui| {
//             ui.label(format!("Mouse Position: ({:.0}, {:.0})", mouse_pos.x, mouse_pos.y));
//         });

//         // Request a repaint for continuous update
//         ctx.request_repaint();
//     }
// }

