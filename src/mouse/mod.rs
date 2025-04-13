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

#[allow(clippy::upper_case_acronyms)]
pub enum MouseClick {
    LEFT,
    RIGHT,
    MIDDLE,
}

#[allow(clippy::upper_case_acronyms)]
pub enum MouseScroll {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub mod mouse_position;
