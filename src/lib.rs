
pub mod safe_mode;
pub mod unsafe_mode;

#[cfg(not(feature = "unsafe-mode"))]
mod mode {
    pub use crate::safe_mode::*;
}

#[cfg(feature = "unsafe-mode")]
mod mode {
    pub use crate::unsafe_mode::*;
}

pub use mode::*;