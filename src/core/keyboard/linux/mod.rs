pub(crate) mod x11;
use x11::X11Keyboard;

use crate::errors::AutoGuiError;

pub enum Keyboard {
    X11(X11Keyboard),
    Wayland,
}

impl Keyboard {
    pub fn get_keymap_key(&self, key: &str) -> Result<&(String, bool), AutoGuiError> {
        match self {
            Keyboard::X11(kb) => kb.get_keymap_key(key),
            Keyboard::Wayland => todo!(),
        }
    }
    pub fn key_down(&self, key: &str) -> Result<(), AutoGuiError> {
        match self {
            Keyboard::X11(kb) => kb.key_down(key),
            Keyboard::Wayland => todo!(),
        }
    }

    pub fn key_up(&self, key: &str) -> Result<(), AutoGuiError> {
        match self {
            Keyboard::X11(kb) => kb.key_up(key),
            Keyboard::Wayland => todo!(),
        }
    }

    /// grabs the value from structs keymap, then converts String to Keysim, and then keysim to Keycode.
    pub unsafe fn get_keycode(&self, key: &str) -> Result<(u32, &bool), AutoGuiError> {
        match self {
            Keyboard::X11(kb) => unsafe { kb.get_keycode(key) },
            Keyboard::Wayland => todo!(),
        }
    }

    /// top level send character function that converts char to keycode and executes send key
    pub fn send_char(&self, key: &char) -> Result<(), AutoGuiError> {
        match self {
            Keyboard::X11(kb) => kb.send_char(key),
            Keyboard::Wayland => todo!(),
        }
    }

    /// similar to send char, but can be string such as return, escape etc
    pub fn send_command(&self, key: &str) -> Result<(), AutoGuiError> {
        match self {
            Keyboard::X11(kb) => kb.send_command(key),
            Keyboard::Wayland => todo!(),
        }
    }

    pub fn send_multi_key(
        &self,
        key_1: &str,
        key_2: &str,
        key_3: Option<String>,
    ) -> Result<(), AutoGuiError> {
        match self {
            Keyboard::X11(kb) => kb.send_multi_key(key_1, key_2, key_3),
            Keyboard::Wayland => todo!(),
        }
    }
}
