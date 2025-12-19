use crate::errors::AutoGuiError;

use super::{MouseClick, MouseScroll};
use std::time::Instant;
use std::{ptr, thread, time::Duration};
pub(crate) mod x11;
pub use x11::X11Mouse;

#[derive(Debug)]
pub enum Mouse {
    X11(X11Mouse),
    Wayland,
}

impl Mouse {
    /// moves mouse to x, y pixel coordinate on screen
    pub fn move_mouse_to_pos(&self, x: i32, y: i32, moving_time: f32) -> Result<(), AutoGuiError> {
        match self {
            Mouse::X11(m) => m.move_mouse_to_pos(x, y, moving_time),
            Mouse::Wayland => todo!(),
        }
    }

    pub fn drag_mouse(&self, x: i32, y: i32, moving_time: f32) -> Result<(), AutoGuiError> {
        match self {
            Mouse::X11(m) => m.drag_mouse(x, y, moving_time),
            Mouse::Wayland => todo!(),
        }
    }

    /// returns x, y pixel coordinate of mouse position
    pub fn get_mouse_position(&self) -> Result<(i32, i32), AutoGuiError> {
        match self {
            Mouse::X11(m) => m.get_mouse_position(),
            Mouse::Wayland => todo!(),
        }
    }

    /// click mouse, either left, right or middle
    pub fn mouse_click(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        match self {
            Mouse::X11(m) => m.mouse_click(button),
            Mouse::Wayland => todo!(),
        }
    }

    pub fn mouse_down(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        match self {
            Mouse::X11(m) => m.mouse_down(button),
            Mouse::Wayland => todo!(),
        }
    }

    pub fn mouse_up(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        match self {
            Mouse::X11(m) => m.mouse_up(button),
            Mouse::Wayland => todo!(),
        }
    }

    pub fn scroll(&self, direction: MouseScroll, intensity: u32) {
        match self {
            Mouse::X11(m) => m.scroll(direction, intensity),
            Mouse::Wayland => todo!(),
        }
    }
}
