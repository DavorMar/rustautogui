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

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseClick {
    LEFT,
    RIGHT,
    MIDDLE,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseScroll {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub mod mouse_position;
