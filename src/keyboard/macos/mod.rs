use core_graphics::event::{CGEvent, CGEventTapLocation, CGKeyCode, KeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use std::collections::HashMap;

use std::thread::sleep;
use std::time::Duration;

pub struct Keyboard {
    keymap: HashMap<String, (u16, bool)>,
}
impl Keyboard {
    pub fn new() -> Self {
        let keymap: HashMap<String, (u16, bool)> = Keyboard::create_keymap();

        Self { keymap: keymap }
    }

    fn press_key(&self, keycode: CGKeyCode) -> Result<(), &'static str> {
        let gc_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let gc_event_source = match gc_event_source {
            Ok(x) => x,
            Err(_) => return Err("Error creating CGEventSource on mouse movement"),
        };
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
        let gc_event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
        let gc_event_source = match gc_event_source {
            Ok(x) => x,
            Err(_) => return Err("Error creating CGEventSource on mouse movement"),
        };
        let event = CGEvent::new_keyboard_event(gc_event_source, keycode, false);
        match event {
            Ok(x) => {
                x.post(CGEventTapLocation::HID);
                sleep(Duration::from_millis(50));
            }
            Err(_) => return Err("Failed to create release key CGkeyboard event"),
        }
        Ok(())
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

    pub fn send_char(&self, key: &char) -> Result<(), &'static str> {
        let char_string = String::from(*key);
        let value = self.keymap.get(&char_string);
        let value = match value {
            Some(x) => x,
            None => return Err("Wrong keyboard key input"),
        };
        let shifted = value.1;
        let value = value.0;
        if shifted {
            self.send_shifted_key(value)?;
        } else {
            self.send_key(value)?;
        };
        Ok(())
    }

    pub fn send_command(&self, key: &String) -> Result<(), &'static str> {
        let value = self.keymap.get(key);
        let value = match value {
            Some(x) => x,
            None => return Err("Wrong keyboard command"),
        };

        self.send_key(value.0)?;
        Ok(())
    }

    pub fn send_multi_key(
        &self,
        key_1: &String,
        key_2: &String,
        key_3: Option<String>,
    ) -> Result<(), &'static str> {
        let value1 = match self.keymap.get(key_1) {
            Some(x) => x,
            None => return Err("False first input in multi key command"),
        }
        .0;
        let value2 = match self.keymap.get(key_2) {
            Some(x) => x,
            None => return Err("False second input in multi key command"),
        }
        .0;

        let mut third_key = false;
        let value3 = match key_3 {
            Some(value) => {
                third_key = true;
                let value3 = match self.keymap.get(&value) {
                    Some(x) => x,
                    None => return Err("False first input in multi key command"),
                };
                value3
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

    fn create_keymap() -> HashMap<String, (u16, bool)> {
        let mut keymap: HashMap<String, (u16, bool)> = HashMap::new();
        keymap.insert(String::from("return"), (KeyCode::RETURN, false));
        keymap.insert(String::from("enter"), (KeyCode::RETURN, false));
        keymap.insert(String::from("tab"), (KeyCode::TAB, false));
        keymap.insert(String::from("space"), (KeyCode::SPACE, false));
        keymap.insert(String::from(" "), (KeyCode::SPACE, false));
        keymap.insert(String::from("delete"), (KeyCode::DELETE, false));
        keymap.insert(String::from("del"), (KeyCode::DELETE, false));
        keymap.insert(String::from("escape"), (KeyCode::ESCAPE, false));
        keymap.insert(String::from("esc"), (KeyCode::ESCAPE, false));
        keymap.insert(String::from("command"), (KeyCode::COMMAND, false));
        keymap.insert(String::from("command_l"), (KeyCode::COMMAND, false));
        keymap.insert(String::from("shift"), (KeyCode::SHIFT, false));
        keymap.insert(String::from("shift_l"), (KeyCode::SHIFT, false));
        keymap.insert(String::from("caps_lock"), (KeyCode::CAPS_LOCK, false));
        keymap.insert(String::from("option"), (KeyCode::OPTION, false));
        keymap.insert(String::from("option_l"), (KeyCode::OPTION, false));
        keymap.insert(String::from("control"), (KeyCode::CONTROL, false));
        keymap.insert(String::from("control_l"), (KeyCode::CONTROL, false));
        keymap.insert(String::from("ctrl"), (KeyCode::CONTROL, false));
        keymap.insert(String::from("command_r"), (KeyCode::RIGHT_COMMAND, false));
        keymap.insert(String::from("shift_r"), (KeyCode::RIGHT_SHIFT, false));
        keymap.insert(String::from("option_r"), (KeyCode::RIGHT_OPTION, false));
        keymap.insert(String::from("control_r"), (KeyCode::RIGHT_CONTROL, false));
        keymap.insert(String::from("function"), (KeyCode::FUNCTION, false));
        keymap.insert(String::from("volumeup"), (KeyCode::VOLUME_UP, false));
        keymap.insert(String::from("volumedown"), (KeyCode::VOLUME_DOWN, false));
        keymap.insert(String::from("volumemute"), (KeyCode::MUTE, false));
        keymap.insert(String::from("F1"), (KeyCode::F1, false));
        keymap.insert(String::from("F2"), (KeyCode::F2, false));
        keymap.insert(String::from("F3"), (KeyCode::F3, false));
        keymap.insert(String::from("F4"), (KeyCode::F4, false));
        keymap.insert(String::from("F5"), (KeyCode::F5, false));
        keymap.insert(String::from("F6"), (KeyCode::F6, false));
        keymap.insert(String::from("F7"), (KeyCode::F7, false));
        keymap.insert(String::from("F8"), (KeyCode::F8, false));
        keymap.insert(String::from("F9"), (KeyCode::F9, false));
        keymap.insert(String::from("F10"), (KeyCode::F10, false));
        keymap.insert(String::from("F11"), (KeyCode::F11, false));
        keymap.insert(String::from("F12"), (KeyCode::F12, false));
        keymap.insert(String::from("F13"), (KeyCode::F13, false));
        keymap.insert(String::from("F14"), (KeyCode::F14, false));
        keymap.insert(String::from("F15"), (KeyCode::F15, false));
        keymap.insert(String::from("F16"), (KeyCode::F16, false));
        keymap.insert(String::from("F17"), (KeyCode::F17, false));
        keymap.insert(String::from("F18"), (KeyCode::F18, false));
        keymap.insert(String::from("F19"), (KeyCode::F19, false));
        keymap.insert(String::from("F20"), (KeyCode::F20, false));
        keymap.insert(String::from("help"), (KeyCode::HELP, false));
        keymap.insert(String::from("home"), (KeyCode::HOME, false));
        keymap.insert(String::from("page_up"), (KeyCode::PAGE_UP, false));
        keymap.insert(String::from("pgup"), (KeyCode::PAGE_UP, false));
        keymap.insert(
            String::from("forward_delete"),
            (KeyCode::FORWARD_DELETE, false),
        );
        keymap.insert(String::from("end"), (KeyCode::END, false));
        keymap.insert(String::from("page_down"), (KeyCode::PAGE_DOWN, false));
        keymap.insert(String::from("pgdn"), (KeyCode::PAGE_DOWN, false));
        keymap.insert(String::from("left_arrow"), (KeyCode::LEFT_ARROW, false));
        keymap.insert(String::from("right_arrow"), (KeyCode::RIGHT_ARROW, false));
        keymap.insert(String::from("down_arrow"), (KeyCode::DOWN_ARROW, false));
        keymap.insert(String::from("up_arrow"), (KeyCode::UP_ARROW, false));
        keymap.insert(String::from("left"), (KeyCode::LEFT_ARROW, false));
        keymap.insert(String::from("right"), (KeyCode::RIGHT_ARROW, false));
        keymap.insert(String::from("down"), (KeyCode::DOWN_ARROW, false));
        keymap.insert(String::from("up"), (KeyCode::UP_ARROW, false));

        keymap.insert(String::from("1"), (18, false));
        keymap.insert(String::from("2"), (19, false));
        keymap.insert(String::from("3"), (20, false));
        keymap.insert(String::from("4"), (21, false));
        keymap.insert(String::from("5"), (23, false));
        keymap.insert(String::from("6"), (22, false));
        keymap.insert(String::from("7"), (26, false));
        keymap.insert(String::from("8"), (28, false));
        keymap.insert(String::from("9"), (25, false));
        keymap.insert(String::from("0"), (29, false));

        keymap.insert(String::from("!"), (18, true));
        keymap.insert(String::from("@"), (19, true));
        keymap.insert(String::from("#"), (20, true));
        keymap.insert(String::from("$"), (21, true));
        keymap.insert(String::from("%"), (23, true));
        keymap.insert(String::from("^"), (22, true));
        keymap.insert(String::from("&"), (26, true));
        keymap.insert(String::from("*"), (28, true));
        keymap.insert(String::from("("), (25, true));
        keymap.insert(String::from(")"), (29, true));

        keymap.insert(String::from("A"), (0, true));
        keymap.insert(String::from("B"), (11, true));
        keymap.insert(String::from("C"), (8, true));
        keymap.insert(String::from("D"), (2, true));
        keymap.insert(String::from("E"), (14, true));
        keymap.insert(String::from("F"), (3, true));
        keymap.insert(String::from("G"), (5, true));
        keymap.insert(String::from("H"), (4, true));
        keymap.insert(String::from("I"), (34, true));
        keymap.insert(String::from("J"), (38, true));
        keymap.insert(String::from("K"), (40, true));
        keymap.insert(String::from("L"), (37, true));
        keymap.insert(String::from("M"), (46, true));
        keymap.insert(String::from("N"), (45, true));
        keymap.insert(String::from("O"), (31, true));
        keymap.insert(String::from("P"), (35, true));
        keymap.insert(String::from("Q"), (12, true));
        keymap.insert(String::from("R"), (15, true));
        keymap.insert(String::from("S"), (1, true));
        keymap.insert(String::from("T"), (17, true));
        keymap.insert(String::from("U"), (32, true));
        keymap.insert(String::from("V"), (9, true));
        keymap.insert(String::from("W"), (13, true));
        keymap.insert(String::from("X"), (7, true));
        keymap.insert(String::from("Y"), (16, true));
        keymap.insert(String::from("Z"), (6, true));

        keymap.insert(String::from("a"), (0, false));
        keymap.insert(String::from("b"), (11, false));
        keymap.insert(String::from("c"), (8, false));
        keymap.insert(String::from("d"), (2, false));
        keymap.insert(String::from("e"), (14, false));
        keymap.insert(String::from("f"), (3, false));
        keymap.insert(String::from("g"), (5, false));
        keymap.insert(String::from("h"), (4, false));
        keymap.insert(String::from("i"), (34, false));
        keymap.insert(String::from("j"), (38, false));
        keymap.insert(String::from("k"), (40, false));
        keymap.insert(String::from("l"), (37, false));
        keymap.insert(String::from("m"), (46, false));
        keymap.insert(String::from("n"), (45, false));
        keymap.insert(String::from("o"), (31, false));
        keymap.insert(String::from("p"), (35, false));
        keymap.insert(String::from("q"), (12, false));
        keymap.insert(String::from("r"), (15, false));
        keymap.insert(String::from("s"), (1, false));
        keymap.insert(String::from("t"), (17, false));
        keymap.insert(String::from("u"), (32, false));
        keymap.insert(String::from("v"), (9, false));
        keymap.insert(String::from("w"), (13, false));
        keymap.insert(String::from("x"), (7, false));
        keymap.insert(String::from("y"), (16, false));
        keymap.insert(String::from("z"), (6, false));

        keymap.insert(String::from("backspace"), (51, false));
        keymap.insert(String::from("insert"), (114, false));
        keymap.insert(String::from("print_screen"), (105, false));
        keymap.insert(String::from("printscreen"), (105, false));
        keymap.insert(String::from("printscrn"), (105, false));
        keymap.insert(String::from("prtsc"), (105, false));
        keymap.insert(String::from("prtscr"), (105, false));
        keymap.insert(String::from("scroll_lock"), (107, false));
        keymap.insert(String::from("pause"), (113, false));
        keymap.insert(String::from("-"), (27, false));
        keymap.insert(String::from("="), (24, false));
        keymap.insert(String::from("["), (33, false));
        keymap.insert(String::from("]"), (30, false));
        keymap.insert(String::from("\\"), (42, false));
        keymap.insert(String::from(";"), (41, false));
        keymap.insert(String::from("'"), (39, false));
        keymap.insert(String::from(","), (43, false));
        keymap.insert(String::from("."), (47, false));

        keymap.insert(String::from("_"), (27, true));
        keymap.insert(String::from("+"), (24, true));
        keymap.insert(String::from("{"), (33, true));
        keymap.insert(String::from("}"), (30, true));
        keymap.insert(String::from("|"), (42, true));
        keymap.insert(String::from(":"), (41, true));
        keymap.insert(String::from("\""), (39, true));
        keymap.insert(String::from("<"), (43, true));
        keymap.insert(String::from(">"), (47, true));
        keymap.insert(String::from("/"), (44, true));
        keymap
    }
}
