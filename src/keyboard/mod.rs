use std::collections::HashMap;

use crate::errors::AutoGuiError;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
use windows::Keyboard;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
use linux::Keyboard;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
use macos::Keyboard;

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn get_keymap_key<'a>(target: &'a Keyboard, key: &str) -> Result<&'a (u16, bool), AutoGuiError> {
    let values = target
        .keymap
        .get(key)
        .ok_or(AutoGuiError::UnSupportedKey(format!(
            "{} key/command is not supported",
            key
        )))?;
    Ok(values)
}

#[cfg(target_os = "linux")]
fn get_keymap_key<'a>(target: &'a Keyboard, key: &str) -> Result<&'a (String, bool), AutoGuiError> {
    let values = target
        .keymap
        .get(key)
        .ok_or(AutoGuiError::UnSupportedKey(format!(
            "{} key/command is not supported",
            key
        )))?;
    Ok(values)
}
