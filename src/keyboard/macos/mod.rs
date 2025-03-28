use core_graphics::event::{CGEvent, CGEventTapLocation, CGKeyCode, KeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use std::collections::HashMap;

use std::thread::sleep;
use std::time::Duration;

use crate::errors::AutoGuiError;

use super::get_keymap_key;
use super::Str;

pub struct Keyboard {
    pub keymap: HashMap<Str, (u16, bool)>,
}
impl Keyboard {
    pub fn new() -> Self {
        let keymap: HashMap<Str, (u16, bool)> = Keyboard::create_keymap();

        Self { keymap }
    }

    fn press_key(&self, keycode: CGKeyCode) -> Result<(), AutoGuiError> {
        let gc_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let gc_event_source = gc_event_source.map_err(|_| {
            AutoGuiError::OSFailure("failed to create CGEvent for key press".to_string())
        })?;
        let event = CGEvent::new_keyboard_event(gc_event_source, keycode, true)
            .map_err(|_| AutoGuiError::OSFailure("Failed creating CGKeyboard event".to_string()))?;
        event.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(50));
        Ok(())
    }

    fn release_key(&self, keycode: CGKeyCode) -> Result<(), AutoGuiError> {
        let gc_event_source =
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
                AutoGuiError::OSFailure(
                    "Error creating CGEventSource on mouse movement".to_string(),
                )
            })?;
        let event = CGEvent::new_keyboard_event(gc_event_source, keycode, false).map_err(|_| {
            AutoGuiError::OSFailure("Failed to create release key CGkeyboard event".to_string())
        })?;
        event.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(50));
        Ok(())
    }

    fn send_key(&self, keycode: CGKeyCode) -> Result<(), AutoGuiError> {
        self.press_key(keycode)?;
        self.release_key(keycode)?;
        Ok(())
    }

    fn send_shifted_key(&self, keycode: CGKeyCode) -> Result<(), AutoGuiError> {
        self.press_key(KeyCode::SHIFT)?;
        self.send_key(keycode)?;
        self.release_key(KeyCode::SHIFT)?;
        Ok(())
    }

    pub fn send_char(&self, key: char) -> Result<(), AutoGuiError> {
        let char_string = String::from(key);
        let (value, shifted) = get_keymap_key(self, &char_string)?;
        if shifted {
            self.send_shifted_key(value)?;
        } else {
            self.send_key(value)?;
        };
        Ok(())
    }

    pub fn send_command(&self, key: &str) -> Result<(), AutoGuiError> {
        let value = crate::keyboard::get_keymap_key(self, key)?;

        self.send_key(value.0)?;
        Ok(())
    }

    pub fn send_multi_key(
        &self,
        key_1: &str,
        key_2: &str,
        key_3: Option<&str>,
    ) -> Result<(), AutoGuiError> {
        let value1 = self
            .keymap
            .get(key_1)
            .ok_or(AutoGuiError::UnSupportedKey(format!(
                "{} key is not supported",
                key_1
            )))?
            .0;
        let value2 = self
            .keymap
            .get(key_2)
            .ok_or(AutoGuiError::UnSupportedKey(format!(
                "{} key is not supported",
                key_2
            )))?
            .0;

        let mut third_key = false;
        let value3 = match key_3 {
            Some(value) => {
                third_key = true;
                self.keymap
                    .get(value)
                    .ok_or(AutoGuiError::UnSupportedKey(format!(
                        "{} key is not supported",
                        value
                    )))?
                    .0
            }
            None => 0,
        };

        self.press_key(value1)?;
        sleep(Duration::from_millis(50));
        self.press_key(value2)?;
        sleep(Duration::from_millis(50));
        if third_key {
            self.press_key(value3)?;
            sleep(Duration::from_millis(50));
            self.release_key(value3)?;
            sleep(Duration::from_millis(50));
        }
        self.release_key(value2)?;
        sleep(Duration::from_millis(50));
        self.release_key(value1)?;
        sleep(Duration::from_millis(50));

        Ok(())
    }

    fn create_keymap() -> HashMap<Str, (u16, bool)> {
        let mut keymap = HashMap::new();
        keymap.insert("return".into(), (KeyCode::RETURN, false));
        keymap.insert("enter".into(), (KeyCode::RETURN, false));
        keymap.insert("tab".into(), (KeyCode::TAB, false));
        keymap.insert("space".into(), (KeyCode::SPACE, false));
        keymap.insert(" ".into(), (KeyCode::SPACE, false));
        keymap.insert("delete".into(), (KeyCode::DELETE, false));
        keymap.insert("del".into(), (KeyCode::DELETE, false));
        keymap.insert("escape".into(), (KeyCode::ESCAPE, false));
        keymap.insert("esc".into(), (KeyCode::ESCAPE, false));
        keymap.insert("command".into(), (KeyCode::COMMAND, false));
        keymap.insert("command_l".into(), (KeyCode::COMMAND, false));
        keymap.insert("shift".into(), (KeyCode::SHIFT, false));
        keymap.insert("shift_l".into(), (KeyCode::SHIFT, false));
        keymap.insert("caps_lock".into(), (KeyCode::CAPS_LOCK, false));
        keymap.insert("option".into(), (KeyCode::OPTION, false));
        keymap.insert("option_l".into(), (KeyCode::OPTION, false));
        keymap.insert("control".into(), (KeyCode::CONTROL, false));
        keymap.insert("control_l".into(), (KeyCode::CONTROL, false));
        keymap.insert("ctrl".into(), (KeyCode::CONTROL, false));
        keymap.insert("command_r".into(), (KeyCode::RIGHT_COMMAND, false));
        keymap.insert("shift_r".into(), (KeyCode::RIGHT_SHIFT, false));
        keymap.insert("option_r".into(), (KeyCode::RIGHT_OPTION, false));
        keymap.insert("control_r".into(), (KeyCode::RIGHT_CONTROL, false));
        keymap.insert("function".into(), (KeyCode::FUNCTION, false));
        keymap.insert("volumeup".into(), (KeyCode::VOLUME_UP, false));
        keymap.insert("volumedown".into(), (KeyCode::VOLUME_DOWN, false));
        keymap.insert("volumemute".into(), (KeyCode::MUTE, false));
        keymap.insert("F1".into(), (KeyCode::F1, false));
        keymap.insert("F2".into(), (KeyCode::F2, false));
        keymap.insert("F3".into(), (KeyCode::F3, false));
        keymap.insert("F4".into(), (KeyCode::F4, false));
        keymap.insert("F5".into(), (KeyCode::F5, false));
        keymap.insert("F6".into(), (KeyCode::F6, false));
        keymap.insert("F7".into(), (KeyCode::F7, false));
        keymap.insert("F8".into(), (KeyCode::F8, false));
        keymap.insert("F9".into(), (KeyCode::F9, false));
        keymap.insert("F10".into(), (KeyCode::F10, false));
        keymap.insert("F11".into(), (KeyCode::F11, false));
        keymap.insert("F12".into(), (KeyCode::F12, false));
        keymap.insert("F13".into(), (KeyCode::F13, false));
        keymap.insert("F14".into(), (KeyCode::F14, false));
        keymap.insert("F15".into(), (KeyCode::F15, false));
        keymap.insert("F16".into(), (KeyCode::F16, false));
        keymap.insert("F17".into(), (KeyCode::F17, false));
        keymap.insert("F18".into(), (KeyCode::F18, false));
        keymap.insert("F19".into(), (KeyCode::F19, false));
        keymap.insert("F20".into(), (KeyCode::F20, false));
        keymap.insert("help".into(), (KeyCode::HELP, false));
        keymap.insert("home".into(), (KeyCode::HOME, false));
        keymap.insert("page_up".into(), (KeyCode::PAGE_UP, false));
        keymap.insert("pgup".into(), (KeyCode::PAGE_UP, false));
        keymap.insert("forward_delete".into(), (KeyCode::FORWARD_DELETE, false));
        keymap.insert("end".into(), (KeyCode::END, false));
        keymap.insert("page_down".into(), (KeyCode::PAGE_DOWN, false));
        keymap.insert("pgdn".into(), (KeyCode::PAGE_DOWN, false));
        keymap.insert("left_arrow".into(), (KeyCode::LEFT_ARROW, false));
        keymap.insert("right_arrow".into(), (KeyCode::RIGHT_ARROW, false));
        keymap.insert("down_arrow".into(), (KeyCode::DOWN_ARROW, false));
        keymap.insert("up_arrow".into(), (KeyCode::UP_ARROW, false));
        keymap.insert("left".into(), (KeyCode::LEFT_ARROW, false));
        keymap.insert("right".into(), (KeyCode::RIGHT_ARROW, false));
        keymap.insert("down".into(), (KeyCode::DOWN_ARROW, false));
        keymap.insert("up".into(), (KeyCode::UP_ARROW, false));

        keymap.insert("1".into(), (18, false));
        keymap.insert("2".into(), (19, false));
        keymap.insert("3".into(), (20, false));
        keymap.insert("4".into(), (21, false));
        keymap.insert("5".into(), (23, false));
        keymap.insert("6".into(), (22, false));
        keymap.insert("7".into(), (26, false));
        keymap.insert("8".into(), (28, false));
        keymap.insert("9".into(), (25, false));
        keymap.insert("0".into(), (29, false));

        keymap.insert("!".into(), (18, true));
        keymap.insert("@".into(), (19, true));
        keymap.insert("#".into(), (20, true));
        keymap.insert("$".into(), (21, true));
        keymap.insert("%".into(), (23, true));
        keymap.insert("^".into(), (22, true));
        keymap.insert("&".into(), (26, true));
        keymap.insert("*".into(), (28, true));
        keymap.insert("(".into(), (25, true));
        keymap.insert(".into()".into(), (29, true));

        keymap.insert("A".into(), (0, true));
        keymap.insert("B".into(), (11, true));
        keymap.insert("C".into(), (8, true));
        keymap.insert("D".into(), (2, true));
        keymap.insert("E".into(), (14, true));
        keymap.insert("F".into(), (3, true));
        keymap.insert("G".into(), (5, true));
        keymap.insert("H".into(), (4, true));
        keymap.insert("I".into(), (34, true));
        keymap.insert("J".into(), (38, true));
        keymap.insert("K".into(), (40, true));
        keymap.insert("L".into(), (37, true));
        keymap.insert("M".into(), (46, true));
        keymap.insert("N".into(), (45, true));
        keymap.insert("O".into(), (31, true));
        keymap.insert("P".into(), (35, true));
        keymap.insert("Q".into(), (12, true));
        keymap.insert("R".into(), (15, true));
        keymap.insert("S".into(), (1, true));
        keymap.insert("T".into(), (17, true));
        keymap.insert("U".into(), (32, true));
        keymap.insert("V".into(), (9, true));
        keymap.insert("W".into(), (13, true));
        keymap.insert("X".into(), (7, true));
        keymap.insert("Y".into(), (16, true));
        keymap.insert("Z".into(), (6, true));

        keymap.insert("a".into(), (0, false));
        keymap.insert("b".into(), (11, false));
        keymap.insert("c".into(), (8, false));
        keymap.insert("d".into(), (2, false));
        keymap.insert("e".into(), (14, false));
        keymap.insert("f".into(), (3, false));
        keymap.insert("g".into(), (5, false));
        keymap.insert("h".into(), (4, false));
        keymap.insert("i".into(), (34, false));
        keymap.insert("j".into(), (38, false));
        keymap.insert("k".into(), (40, false));
        keymap.insert("l".into(), (37, false));
        keymap.insert("m".into(), (46, false));
        keymap.insert("n".into(), (45, false));
        keymap.insert("o".into(), (31, false));
        keymap.insert("p".into(), (35, false));
        keymap.insert("q".into(), (12, false));
        keymap.insert("r".into(), (15, false));
        keymap.insert("s".into(), (1, false));
        keymap.insert("t".into(), (17, false));
        keymap.insert("u".into(), (32, false));
        keymap.insert("v".into(), (9, false));
        keymap.insert("w".into(), (13, false));
        keymap.insert("x".into(), (7, false));
        keymap.insert("y".into(), (16, false));
        keymap.insert("z".into(), (6, false));

        keymap.insert("backspace".into(), (51, false));
        keymap.insert("insert".into(), (114, false));
        keymap.insert("print_screen".into(), (105, false));
        keymap.insert("printscreen".into(), (105, false));
        keymap.insert("printscrn".into(), (105, false));
        keymap.insert("prtsc".into(), (105, false));
        keymap.insert("prtscr".into(), (105, false));
        keymap.insert("scroll_lock".into(), (107, false));
        keymap.insert("pause".into(), (113, false));
        keymap.insert("-".into(), (27, false));
        keymap.insert("=".into(), (24, false));
        keymap.insert("[".into(), (33, false));
        keymap.insert("]".into(), (30, false));
        keymap.insert("\\".into(), (42, false));
        keymap.insert(";".into(), (41, false));
        keymap.insert("'".into(), (39, false));
        keymap.insert(",".into(), (43, false));
        keymap.insert(".".into(), (47, false));

        keymap.insert("_".into(), (27, true));
        keymap.insert("+".into(), (24, true));
        keymap.insert("{".into(), (33, true));
        keymap.insert("}".into(), (30, true));
        keymap.insert("|".into(), (42, true));
        keymap.insert(":".into(), (41, true));
        keymap.insert("\"".into(), (39, true));
        keymap.insert("<".into(), (43, true));
        keymap.insert(">".into(), (47, true));
        keymap.insert("/".into(), (44, true));
        keymap
    }
}
