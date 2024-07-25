use core_graphics::event::{
    CGEvent, CGEventType, CGEventTapLocation, CGKeyCode, KeyCode
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use std::collections::HashMap;

use std::hash::Hash;
use std::thread::sleep;
use std::time::Duration;




pub struct Keyboard {
    keymap: HashMap <String,u16>,
}
impl Keyboard {
    pub fn new() -> Self {
        let mut keymap = Keyboard::create_keymap();
       
        Self { keymap:keymap}
    }
    fn press_key(&self, keycode: CGKeyCode) {
        let event = CGEvent::new_keyboard_event(
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap(),
            keycode,
            true,
        ).unwrap();
        event.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(50));
    }
    
    fn release_key(&self, keycode: CGKeyCode) {
        let event = CGEvent::new_keyboard_event(
            CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap(),
            keycode,
            false,
        ).unwrap();
        event.post(CGEventTapLocation::HID);
        sleep(Duration::from_millis(50));
    }
    
    fn send_key(&self, keycode: CGKeyCode) {
        self.press_key(keycode);
        self.release_key(keycode);
    }

    fn send_shifted_key(&self, keycode:CGKeyCode) {
        self.press_key(KeyCode::SHIFT);
        self.send_key(keycode);
        self.release_key(KeyCode::SHIFT)
    }

    pub fn send_char(&self, key:&char, shifted:&bool) {
        let char_string = String::from(*key);
        let value = self.keymap.get(&char_string);
        match value {
            Some(_) => (),
            None => {
                return
            }
        }
        if *shifted {
            self.send_shifted_key(*value.unwrap());
        } else {
            self.send_key(*value.unwrap());
        }
    }

    pub fn send_command(&self, key:&String) {
        let value = self.keymap.get(key);
        self.send_key(*value.expect("Wrong input key"));
    }


    fn create_keymap() -> HashMap<String, u16> {
        let mut keymap = HashMap::new();
        keymap.insert(String::from("return"), KeyCode::RETURN);
        keymap.insert(String::from("tab"), KeyCode::TAB);
        keymap.insert(String::from("space"), KeyCode::SPACE);
        keymap.insert(String::from("delete"), KeyCode::DELETE);
        keymap.insert(String::from("escape"), KeyCode::ESCAPE);
        keymap.insert(String::from("command"), KeyCode::COMMAND);
        keymap.insert(String::from("shift"), KeyCode::SHIFT);
        keymap.insert(String::from("caps_lock"), KeyCode::CAPS_LOCK);
        keymap.insert(String::from("option"), KeyCode::OPTION);
        keymap.insert(String::from("control"), KeyCode::CONTROL);
        keymap.insert(String::from("command_r"), KeyCode::RIGHT_COMMAND);
        keymap.insert(String::from("shift_r"), KeyCode::RIGHT_SHIFT);
        keymap.insert(String::from("option_r"), KeyCode::RIGHT_OPTION);
        keymap.insert(String::from("control_r"), KeyCode::RIGHT_CONTROL);
        keymap.insert(String::from("function"), KeyCode::FUNCTION);
        keymap.insert(String::from("vol_up"), KeyCode::VOLUME_UP);
        keymap.insert(String::from("vol_down"), KeyCode::VOLUME_DOWN);
        keymap.insert(String::from("mute"), KeyCode::MUTE);
        keymap.insert(String::from("F1"), KeyCode::F1);
        keymap.insert(String::from("F2"), KeyCode::F2);
        keymap.insert(String::from("F3"), KeyCode::F3);
        keymap.insert(String::from("F4"), KeyCode::F4);
        keymap.insert(String::from("F5"), KeyCode::F5);
        keymap.insert(String::from("F6"), KeyCode::F6);
        keymap.insert(String::from("F7"), KeyCode::F7);
        keymap.insert(String::from("F8"), KeyCode::F8);
        keymap.insert(String::from("F9"), KeyCode::F9);
        keymap.insert(String::from("F10"), KeyCode::F10);
        keymap.insert(String::from("F11"), KeyCode::F11);
        keymap.insert(String::from("F12"), KeyCode::F12);
        keymap.insert(String::from("F13"), KeyCode::F13);
        keymap.insert(String::from("F14"), KeyCode::F14);
        keymap.insert(String::from("F15"), KeyCode::F15);
        keymap.insert(String::from("F16"), KeyCode::F16);
        keymap.insert(String::from("F17"), KeyCode::F17);
        keymap.insert(String::from("F18"), KeyCode::F18);
        keymap.insert(String::from("F19"), KeyCode::F19);
        keymap.insert(String::from("F20"), KeyCode::F20);
        keymap.insert(String::from("help"), KeyCode::HELP);
        keymap.insert(String::from("home"), KeyCode::HOME);
        keymap.insert(String::from("page_up"), KeyCode::PAGE_UP);
        keymap.insert(String::from("forward_delete"), KeyCode::FORWARD_DELETE);
        keymap.insert(String::from("end"), KeyCode::END);
        keymap.insert(String::from("page_down"), KeyCode::PAGE_DOWN);
        keymap.insert(String::from("left_arrow"), KeyCode::LEFT_ARROW);
        keymap.insert(String::from("right_arrow"), KeyCode::RIGHT_ARROW);
        keymap.insert(String::from("down_arrow"), KeyCode::DOWN_ARROW);
        keymap.insert(String::from("up_arrow"), KeyCode::UP_ARROW);
        keymap.insert(String::from("1"), 18);
        keymap.insert(String::from("2"), 19);
        keymap.insert(String::from("3"), 20);
        keymap.insert(String::from("4"), 21);
        keymap.insert(String::from("5"), 23);
        keymap.insert(String::from("6"), 22);
        keymap.insert(String::from("7"), 26);
        keymap.insert(String::from("8"), 28);
        keymap.insert(String::from("9"), 25);
        keymap.insert(String::from("0"), 29);
        keymap.insert(String::from("a"), 0);
        keymap.insert(String::from("b"), 11);
        keymap.insert(String::from("c"), 8);
        keymap.insert(String::from("d"), 2);
        keymap.insert(String::from("e"), 14);
        keymap.insert(String::from("f"), 3);
        keymap.insert(String::from("g"), 5);
        keymap.insert(String::from("h"), 4);
        keymap.insert(String::from("i"), 34);
        keymap.insert(String::from("j"), 38);
        keymap.insert(String::from("k"), 40);
        keymap.insert(String::from("l"), 37);
        keymap.insert(String::from("m"), 46);
        keymap.insert(String::from("n"), 45);
        keymap.insert(String::from("o"), 31);
        keymap.insert(String::from("p"), 35);
        keymap.insert(String::from("q"), 12);
        keymap.insert(String::from("r"), 15);
        keymap.insert(String::from("s"), 1);
        keymap.insert(String::from("t"), 17);
        keymap.insert(String::from("u"), 32);
        keymap.insert(String::from("v"), 9);
        keymap.insert(String::from("w"), 13);
        keymap.insert(String::from("x"), 7);
        keymap.insert(String::from("y"), 16);
        keymap.insert(String::from("z"), 6);
        keymap
    }

}

