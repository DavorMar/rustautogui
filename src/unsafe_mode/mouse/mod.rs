#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;


#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub use windows as platform;

#[cfg(target_os = "macos")]
pub use macos as platform;

#[cfg(target_os = "linux")]
pub use linux as platform;


pub enum MouseClick {
    LEFT,
    RIGHT,
    MIDDLE
}

pub enum MouseScroll {
    UP,
    DOWN,
}

pub mod mouse_position;