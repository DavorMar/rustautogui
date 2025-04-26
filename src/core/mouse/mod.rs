#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub use windows::Mouse;

#[cfg(target_os = "macos")]
pub use macos::Mouse;

#[cfg(target_os = "linux")]
pub use linux::Mouse;

pub enum MouseClick {
    LEFT,
    RIGHT,
    MIDDLE,
}

pub enum MouseScroll {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub mod mouse_position;
