use core_graphics::event::{CGEvent, CGEventTapLocation, CGKeyCode, KeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use std::collections::HashMap;

use std::thread::sleep;
use std::time::Duration;

pub struct Keyboard {
    keymap: HashMap<&'static str, (u16, bool)>,
}
impl Keyboard {
    pub fn new() -> Self {
        let keymap = Keyboard::create_keymap();

        Self { keymap }
    }

    fn press_key(&self, keycode: CGKeyCode) -> Result<(), &'static str> {
        let gc_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| "Error creating CGEventSource on mouse movement")?;
        let event = CGEvent::new_keyboard_event(gc_event_source, keycode, true);
        match event {
            Ok(x) => {
                x.post(CGEventTapLocation::HID);
                sleep(Duration::from_millis(50));
            }
            Err(_) => return Err("Failed creatomg CGKeyboard event"),
        }

        Ok(())
    }

    fn release_key(&self, keycode: CGKeyCode) -> Result<(), &'static str> {
        let gc_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| "Error creating CGEventSource on mouse movement")?;
        let event = CGEvent::new_keyboard_event(gc_event_source, keycode, false);
        match event {
            Ok(x) => {
                x.post(CGEventTapLocation::HID);
                Ok(sleep(Duration::from_millis(50)))
            }
            Err(_) => Err("Failed to create release key CGkeyboard event"),
        }
    }

    fn send_key(&self, keycode: CGKeyCode) -> Result<(), &'static str> {
        self.press_key(keycode)?;
        self.release_key(keycode)?;
        Ok(())
    }

    fn send_shifted_key(&self, keycode: CGKeyCode) -> Result<(), &'static str> {
        self.press_key(KeyCode::SHIFT)?;
        self.send_key(keycode)?;
        self.release_key(KeyCode::SHIFT)?;
        Ok(())
    }

    pub fn send_char(&self, key: char) -> Result<(), &'static str> {
        let char_string = String::from(key);
        let &(value, shifted) = self
            .keymap
            .get(&char_string.as_str())
            .ok_or("Wrong keyboard key input")?;
        if shifted {
            self.send_shifted_key(value)?;
        } else {
            self.send_key(value)?;
        };
        Ok(())
    }

    pub fn send_command(&self, key: &str) -> Result<(), &'static str> {
        let value = self.keymap.get(key).ok_or("Wrong keyboard command")?;

        self.send_key(value.0)?;
        Ok(())
    }

    pub fn send_multi_key(
        &self,
        key_1: &str,
        key_2: &str,
        key_3: Option<&str>,
    ) -> Result<(), &'static str> {
        let value1 = self
            .keymap
            .get(key_1)
            .ok_or("False first input in multi key command")?
            .0;
        let value2 = self
            .keymap
            .get(key_2)
            .ok_or("False second input in multi key command")?
            .0;

        let mut third_key = false;
        let value3 = match key_3 {
            Some(value) => {
                third_key = true;
                self.keymap
                    .get(value)
                    .ok_or("False first input in multi key command")?
            }
            None => &(0, false),
        }
        .0;

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

    fn create_keymap() -> HashMap<&'static str, (u16, bool)> {
        let mut keymap = HashMap::with_capacity(180);
        keymap.insert("return", (KeyCode::RETURN, false));
        keymap.insert("enter", (KeyCode::RETURN, false));
        keymap.insert("tab", (KeyCode::TAB, false));
        keymap.insert("space", (KeyCode::SPACE, false));
        keymap.insert(" ", (KeyCode::SPACE, false));
        keymap.insert("delete", (KeyCode::DELETE, false));
        keymap.insert("del", (KeyCode::DELETE, false));
        keymap.insert("escape", (KeyCode::ESCAPE, false));
        keymap.insert("esc", (KeyCode::ESCAPE, false));
        keymap.insert("command", (KeyCode::COMMAND, false));
        keymap.insert("command_l", (KeyCode::COMMAND, false));
        keymap.insert("shift", (KeyCode::SHIFT, false));
        keymap.insert("shift_l", (KeyCode::SHIFT, false));
        keymap.insert("caps_lock", (KeyCode::CAPS_LOCK, false));
        keymap.insert("option", (KeyCode::OPTION, false));
        keymap.insert("option_l", (KeyCode::OPTION, false));
        keymap.insert("control", (KeyCode::CONTROL, false));
        keymap.insert("control_l", (KeyCode::CONTROL, false));
        keymap.insert("ctrl", (KeyCode::CONTROL, false));
        keymap.insert("command_r", (KeyCode::RIGHT_COMMAND, false));
        keymap.insert("shift_r", (KeyCode::RIGHT_SHIFT, false));
        keymap.insert("option_r", (KeyCode::RIGHT_OPTION, false));
        keymap.insert("control_r", (KeyCode::RIGHT_CONTROL, false));
        keymap.insert("function", (KeyCode::FUNCTION, false));
        keymap.insert("volumeup", (KeyCode::VOLUME_UP, false));
        keymap.insert("volumedown", (KeyCode::VOLUME_DOWN, false));
        keymap.insert("volumemute", (KeyCode::MUTE, false));
        keymap.insert("F1", (KeyCode::F1, false));
        keymap.insert("F2", (KeyCode::F2, false));
        keymap.insert("F3", (KeyCode::F3, false));
        keymap.insert("F4", (KeyCode::F4, false));
        keymap.insert("F5", (KeyCode::F5, false));
        keymap.insert("F6", (KeyCode::F6, false));
        keymap.insert("F7", (KeyCode::F7, false));
        keymap.insert("F8", (KeyCode::F8, false));
        keymap.insert("F9", (KeyCode::F9, false));
        keymap.insert("F10", (KeyCode::F10, false));
        keymap.insert("F11", (KeyCode::F11, false));
        keymap.insert("F12", (KeyCode::F12, false));
        keymap.insert("F13", (KeyCode::F13, false));
        keymap.insert("F14", (KeyCode::F14, false));
        keymap.insert("F15", (KeyCode::F15, false));
        keymap.insert("F16", (KeyCode::F16, false));
        keymap.insert("F17", (KeyCode::F17, false));
        keymap.insert("F18", (KeyCode::F18, false));
        keymap.insert("F19", (KeyCode::F19, false));
        keymap.insert("F20", (KeyCode::F20, false));
        keymap.insert("help", (KeyCode::HELP, false));
        keymap.insert("home", (KeyCode::HOME, false));
        keymap.insert("page_up", (KeyCode::PAGE_UP, false));
        keymap.insert("pgup", (KeyCode::PAGE_UP, false));
        keymap.insert("forward_delete", (KeyCode::FORWARD_DELETE, false));
        keymap.insert("end", (KeyCode::END, false));
        keymap.insert("page_down", (KeyCode::PAGE_DOWN, false));
        keymap.insert("pgdn", (KeyCode::PAGE_DOWN, false));
        keymap.insert("left_arrow", (KeyCode::LEFT_ARROW, false));
        keymap.insert("right_arrow", (KeyCode::RIGHT_ARROW, false));
        keymap.insert("down_arrow", (KeyCode::DOWN_ARROW, false));
        keymap.insert("up_arrow", (KeyCode::UP_ARROW, false));
        keymap.insert("left", (KeyCode::LEFT_ARROW, false));
        keymap.insert("right", (KeyCode::RIGHT_ARROW, false));
        keymap.insert("down", (KeyCode::DOWN_ARROW, false));
        keymap.insert("up", (KeyCode::UP_ARROW, false));

        keymap.insert("1", (18, false));
        keymap.insert("2", (19, false));
        keymap.insert("3", (20, false));
        keymap.insert("4", (21, false));
        keymap.insert("5", (23, false));
        keymap.insert("6", (22, false));
        keymap.insert("7", (26, false));
        keymap.insert("8", (28, false));
        keymap.insert("9", (25, false));
        keymap.insert("0", (29, false));

        keymap.insert("!", (18, true));
        keymap.insert("@", (19, true));
        keymap.insert("#", (20, true));
        keymap.insert("$", (21, true));
        keymap.insert("%", (23, true));
        keymap.insert("^", (22, true));
        keymap.insert("&", (26, true));
        keymap.insert("*", (28, true));
        keymap.insert("(", (25, true));
        keymap.insert(")", (29, true));

        keymap.insert("A", (0, true));
        keymap.insert("B", (11, true));
        keymap.insert("C", (8, true));
        keymap.insert("D", (2, true));
        keymap.insert("E", (14, true));
        keymap.insert("F", (3, true));
        keymap.insert("G", (5, true));
        keymap.insert("H", (4, true));
        keymap.insert("I", (34, true));
        keymap.insert("J", (38, true));
        keymap.insert("K", (40, true));
        keymap.insert("L", (37, true));
        keymap.insert("M", (46, true));
        keymap.insert("N", (45, true));
        keymap.insert("O", (31, true));
        keymap.insert("P", (35, true));
        keymap.insert("Q", (12, true));
        keymap.insert("R", (15, true));
        keymap.insert("S", (1, true));
        keymap.insert("T", (17, true));
        keymap.insert("U", (32, true));
        keymap.insert("V", (9, true));
        keymap.insert("W", (13, true));
        keymap.insert("X", (7, true));
        keymap.insert("Y", (16, true));
        keymap.insert("Z", (6, true));

        keymap.insert("a", (0, false));
        keymap.insert("b", (11, false));
        keymap.insert("c", (8, false));
        keymap.insert("d", (2, false));
        keymap.insert("e", (14, false));
        keymap.insert("f", (3, false));
        keymap.insert("g", (5, false));
        keymap.insert("h", (4, false));
        keymap.insert("i", (34, false));
        keymap.insert("j", (38, false));
        keymap.insert("k", (40, false));
        keymap.insert("l", (37, false));
        keymap.insert("m", (46, false));
        keymap.insert("n", (45, false));
        keymap.insert("o", (31, false));
        keymap.insert("p", (35, false));
        keymap.insert("q", (12, false));
        keymap.insert("r", (15, false));
        keymap.insert("s", (1, false));
        keymap.insert("t", (17, false));
        keymap.insert("u", (32, false));
        keymap.insert("v", (9, false));
        keymap.insert("w", (13, false));
        keymap.insert("x", (7, false));
        keymap.insert("y", (16, false));
        keymap.insert("z", (6, false));

        keymap.insert("backspace", (51, false));
        keymap.insert("insert", (114, false));
        keymap.insert("print_screen", (105, false));
        keymap.insert("printscreen", (105, false));
        keymap.insert("printscrn", (105, false));
        keymap.insert("prtsc", (105, false));
        keymap.insert("prtscr", (105, false));
        keymap.insert("scroll_lock", (107, false));
        keymap.insert("pause", (113, false));
        keymap.insert("-", (27, false));
        keymap.insert("=", (24, false));
        keymap.insert("[", (33, false));
        keymap.insert("]", (30, false));
        keymap.insert("\\", (42, false));
        keymap.insert(";", (41, false));
        keymap.insert("'", (39, false));
        keymap.insert(",", (43, false));
        keymap.insert(".", (47, false));

        keymap.insert("_", (27, true));
        keymap.insert("+", (24, true));
        keymap.insert("{", (33, true));
        keymap.insert("}", (30, true));
        keymap.insert("|", (42, true));
        keymap.insert(":", (41, true));
        keymap.insert("\"", (39, true));
        keymap.insert("<", (43, true));
        keymap.insert(">", (47, true));
        keymap.insert("/", (44, true));
        keymap
    }
}
