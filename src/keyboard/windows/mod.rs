use crate::errors::AutoGuiError;
use crate::keyboard::get_keymap_key;
use std::{collections::HashMap, mem::size_of, thread::sleep, time::Duration};
use winapi::um::wingdi::SRCAND;
use winapi::um::winuser::{MapVirtualKeyW, MAPVK_VK_TO_VSC};
use winapi::um::winuser::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VK_CONTROL, VK_MENU,
    VK_SHIFT,
};

use super::Str;

/// main struct for interacting with keyboard. Keymap is generated upon intialization.
pub struct Keyboard {
    pub keymap: HashMap<Str, (u16, bool)>,
}
impl Keyboard {
    /// create new keyboard instance.
    pub fn new() -> Keyboard {
        let keyset = Keyboard::create_keymap();
        Keyboard { keymap: keyset }
    }

    unsafe fn key_down(scan_code: u16) {
        let mut input: INPUT = std::mem::zeroed();
        input.type_ = INPUT_KEYBOARD;
        {
            let ki = input.u.ki_mut();
            if scan_code == VK_SHIFT as u16
                || scan_code == VK_CONTROL as u16
                || scan_code == VK_MENU as u16
            {
                ki.wVk = scan_code; // Use virtual key code for Shift, Control, and Alt
                ki.wScan = 0;
                ki.dwFlags = 0; // No KEYEVENTF_SCANCODE flag for virtual key
            } else {
                let scan_code = MapVirtualKeyW(scan_code as u32, MAPVK_VK_TO_VSC) as u16;
                ki.wVk = 0;
                ki.wScan = scan_code;
                ki.dwFlags = KEYEVENTF_SCANCODE;
            }
            ki.time = 0;
            ki.dwExtraInfo = 0;
        }

        // Send key press
        SendInput(1, &mut input, size_of::<INPUT>() as i32);
    }

    unsafe fn key_up(scan_code: u16) {
        let mut input: INPUT = std::mem::zeroed();
        input.type_ = INPUT_KEYBOARD;
        {
            let ki = input.u.ki_mut();
            if scan_code == VK_SHIFT as u16
                || scan_code == VK_CONTROL as u16
                || scan_code == VK_MENU as u16
            {
                ki.wVk = scan_code; // Use virtual key code for Shift, Control, and Alt
                ki.wScan = 0;
                ki.dwFlags = KEYEVENTF_KEYUP; // No KEYEVENTF_SCANCODE flag for virtual key
            } else {
                let scan_code = MapVirtualKeyW(scan_code as u32, MAPVK_VK_TO_VSC) as u16;
                ki.wVk = 0;
                ki.wScan = scan_code;
                ki.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
            }
            ki.time = 0;
            ki.dwExtraInfo = 0;
        }

        // Release key
        SendInput(1, &mut input, size_of::<INPUT>() as i32);
    }

    /// executes press down of a key, then press up.
    pub fn send_key(scan_code: u16) {
        unsafe {
            Keyboard::key_down(scan_code);
            Keyboard::key_up(scan_code);
        }
    }

    /// executes press down of shift key, press down and press up for desired key, then press up of shift key
    pub fn send_shifted_key(scan_code: u16) {
        unsafe {
            Keyboard::key_down(0x10); // shift press
            sleep(Duration::from_micros(50));
            // send key
            Keyboard::send_key(scan_code);
            sleep(Duration::from_micros(50));

            Keyboard::key_up(0x10); // shift release
            sleep(Duration::from_micros(50));
        }
    }

    /// Function used when sending input as string. All characters need to be part of the key map, described in Keyboard_commands.md
    /// For each character in a string, Keyboard::send_key() is executed. If the character requires a shift key,
    /// Keyboard::send_shifted_key is executed
    pub fn send_char(&self, key: char) -> Result<(), AutoGuiError> {
        let char_string = String::from(key);
        let (value, shifted) = get_keymap_key(self, &char_string)?;

        if shifted {
            Keyboard::send_shifted_key(value);
        } else {
            Keyboard::send_key(value);
        }
        Ok(())
    }

    /// Function used when sending commands like "return" or "escape"
    pub fn send_command(&self, key: &str) -> Result<(), AutoGuiError> {
        let (value, _) = get_keymap_key(self, key)?;
        Keyboard::send_key(value);
        Ok(())
    }

    pub fn send_multi_key(
        &self,
        key_1: &str,
        key_2: &str,
        key_3: Option<&str>,
    ) -> Result<(), AutoGuiError> {
        let (value_1, _) = get_keymap_key(self, key_1)?;
        let (value_2, _) = get_keymap_key(self, key_2)?;

        let mut third_key = false;
        let value_3 = match key_3 {
            Some(value) => {
                third_key = true;
                let (value_, _) = get_keymap_key(self, value)?;
                value_
            }
            None => 0,
        };

        unsafe {
            Keyboard::key_down(value_1);
            Keyboard::key_down(value_2);
            if third_key {
                Keyboard::key_down(value_3);
                Keyboard::key_up(value_3);
            }
            Keyboard::key_up(value_2);
            Keyboard::key_up(value_1);
        }
        Ok(())
    }

    /// mapping made so  bigger variety of strings can be used when sending string as input.
    /// for instance, instead of neccessity of sending "period", we can send ".". This means when sending a
    /// string like url test.hr we dont need to send test, then send period, then send hr
    pub fn create_keymap() -> HashMap<Str, (u16, bool)> {
        let mut key_map = HashMap::new();
        // Inserting key mappings
        key_map.insert("backspace".into(), (0x08, false)); // VK_BACK
        key_map.insert("super".into(), (0x5B, false)); // VK_LWIN
        key_map.insert("tab".into(), (0x09, false)); // VK_TAB
        key_map.insert("clear".into(), (0x0c, false)); // VK_CLEAR
        key_map.insert("enter".into(), (0x0d, false)); // VK_RETURN
        key_map.insert("return".into(), (0x0d, false)); // VK_RETURN
        key_map.insert("shift_l".into(), (0x10, false)); // VK_SHIFT
        key_map.insert("ctrl".into(), (0x11, false)); // VK_CONTROL
        key_map.insert("alt".into(), (0x12, false)); // VK_MENU
        key_map.insert("pause".into(), (0x13, false)); // VK_PAUSE
        key_map.insert("caps_lock".into(), (0x14, false)); // VK_CAPITAL
        key_map.insert("kana".into(), (0x15, false)); // VK_KANA
        key_map.insert("hanguel".into(), (0x15, false)); // VK_HANGUEL
        key_map.insert("hangul".into(), (0x15, false)); // VK_HANGUL
        key_map.insert("junja".into(), (0x17, false)); // VK_JUNJA
        key_map.insert("final".into(), (0x18, false)); // VK_FINAL
        key_map.insert("hanja".into(), (0x19, false)); // VK_HANJA
        key_map.insert("kanji".into(), (0x19, false)); // VK_KANJI
        key_map.insert("esc".into(), (0x1b, false)); // VK_ESCAPE
        key_map.insert("escape".into(), (0x1b, false)); // VK_ESCAPE
        key_map.insert("convert".into(), (0x1c, false)); // VK_CONVERT
        key_map.insert("nonconvert".into(), (0x1d, false)); // VK_NONCONVERT
        key_map.insert("accept".into(), (0x1e, false)); // VK_ACCEPT
        key_map.insert("modechange".into(), (0x1f, false)); // VK_MODECHANGE
        key_map.insert(" ".into(), (0x20, false)); // VK_SPACE
        key_map.insert("space".into(), (0x20, false)); // VK_SPACE
        key_map.insert("pgup".into(), (0x21, false)); // VK_PRIOR
        key_map.insert("pgdn".into(), (0x22, false)); // VK_NEXT
        key_map.insert("page_up".into(), (0x21, false)); // VK_PRIOR
        key_map.insert("page_down".into(), (0x22, false)); // VK_NEXT
        key_map.insert("end".into(), (0x23, false)); // VK_END
        key_map.insert("home".into(), (0x24, false)); // VK_HOME
        key_map.insert("left_arrow".into(), (0x25, false)); // VK_LEFT
        key_map.insert("up_arrow".into(), (0x26, false)); // VK_UP
        key_map.insert("right_arrow".into(), (0x27, false)); // VK_RIGHT
        key_map.insert("down_arrow".into(), (0x28, false)); // VK_DOWN

        key_map.insert("left".into(), (0x25, false)); // VK_LEFT
        key_map.insert("up".into(), (0x26, false)); // VK_UP
        key_map.insert("right".into(), (0x27, false)); // VK_RIGHT
        key_map.insert("down".into(), (0x28, false)); // VK_DOWN

        key_map.insert("select".into(), (0x29, false)); // VK_SELECT
        key_map.insert("print".into(), (0x2a, false)); // VK_PRINT
        key_map.insert("execute".into(), (0x2b, false)); // VK_EXECUTE
        key_map.insert("prtsc".into(), (0x2c, false)); // VK_SNAPSHOT
        key_map.insert("prtscr".into(), (0x2c, false)); // VK_SNAPSHOT
        key_map.insert("prntscrn".into(), (0x2c, false)); // VK_SNAPSHOT
        key_map.insert("printscreen".into(), (0x2c, false)); // VK_SNAPSHOT
        key_map.insert("insert".into(), (0x2d, false)); // VK_INSERT
        key_map.insert("del".into(), (0x2e, false)); // VK_DELETE
        key_map.insert("delete".into(), (0x2e, false)); // VK_DELETE
        key_map.insert("help".into(), (0x2f, false)); // VK_HELP
        key_map.insert("win".into(), (0x5b, false)); // VK_LWIN
        key_map.insert("winleft".into(), (0x5b, false)); // VK_LWIN
        key_map.insert("win_l".into(), (0x5b, false)); // VK_LWIN
        key_map.insert("super".into(), (0x5b, false)); // VK_LWIN
        key_map.insert("super_l".into(), (0x5b, false)); // VK_LWIN
        key_map.insert("winright".into(), (0x5c, false)); // VK_RWIN
        key_map.insert("win_r".into(), (0x5c, false)); // VK_RWIN
        key_map.insert("super_r".into(), (0x5c, false)); // VK_RWIN
        key_map.insert("apps".into(), (0x5d, false)); // VK_APPS
        key_map.insert("sleep".into(), (0x5f, false)); // VK_SLEEP
        key_map.insert("num0".into(), (0x60, false)); // VK_NUMPAD0
        key_map.insert("num1".into(), (0x61, false)); // VK_NUMPAD1
        key_map.insert("num2".into(), (0x62, false)); // VK_NUMPAD2
        key_map.insert("num3".into(), (0x63, false)); // VK_NUMPAD3
        key_map.insert("num4".into(), (0x64, false)); // VK_NUMPAD4
        key_map.insert("num5".into(), (0x65, false)); // VK_NUMPAD5
        key_map.insert("num6".into(), (0x66, false)); // VK_NUMPAD6
        key_map.insert("num7".into(), (0x67, false)); // VK_NUMPAD7
        key_map.insert("num8".into(), (0x68, false)); // VK_NUMPAD8
        key_map.insert("num9".into(), (0x69, false)); // VK_NUMPAD9
        key_map.insert("*".into(), (0x6a, false)); //,false) VK_MULTIPLY
        key_map.insert("+".into(), (0x6b, false)); //,false) VK_ADD
        key_map.insert("=".into(), (0xBB, false)); //,false) VK_OEM plus

        key_map.insert("separator".into(), (0x6c, true)); // VK_SEPARATOR
        key_map.insert("-".into(), (0xBD, false)); //,false) VK_SUBTRACT
        key_map.insert("_".into(), (0xBD, true)); //,false) VK_SUBTRACT
        key_map.insert(".".into(), (0xBE, false)); // VK_OEM_PERIOD
        key_map.insert(",".into(), (0xBC, false)); // VK_OEM_COMMA
        key_map.insert(">".into(), (0xBE, true)); // VK_OEM_PERIOD
        key_map.insert("<".into(), (0xBC, true)); // VK_OEM_COMMA
        key_map.insert("/".into(), (0x6f, false)); // VK_DIVIDE
        key_map.insert("?".into(), (0x6f, true)); // VK_DIVIDE
        key_map.insert("f1".into(), (0x70, false)); // VK_F1
        key_map.insert("f2".into(), (0x71, false)); // VK_F2
        key_map.insert("f3".into(), (0x72, false)); // VK_F3
        key_map.insert("f4".into(), (0x73, false)); // VK_F4
        key_map.insert("f5".into(), (0x74, false)); // VK_F5
        key_map.insert("f6".into(), (0x75, false)); // VK_F6
        key_map.insert("f7".into(), (0x76, false)); // VK_F7
        key_map.insert("f8".into(), (0x77, false)); // VK_F8
        key_map.insert("f9".into(), (0x78, false)); // VK_F9
        key_map.insert("f10".into(), (0x79, false)); // VK_F10
        key_map.insert("f11".into(), (0x7a, false)); // VK_F11
        key_map.insert("f12".into(), (0x7b, false)); // VK_F12
        key_map.insert("f13".into(), (0x7c, false)); // VK_F13
        key_map.insert("f14".into(), (0x7d, false)); // VK_F14
        key_map.insert("f15".into(), (0x7e, false)); // VK_F15
        key_map.insert("f16".into(), (0x7f, false)); // VK_F16
        key_map.insert("f17".into(), (0x80, false)); // VK_F17
        key_map.insert("f18".into(), (0x81, false)); // VK_F18
        key_map.insert("f19".into(), (0x82, false)); // VK_F19
        key_map.insert("f20".into(), (0x83, false)); // VK_F20
        key_map.insert("f21".into(), (0x84, false)); // VK_F21
        key_map.insert("f22".into(), (0x85, false)); // VK_F22
        key_map.insert("f23".into(), (0x86, false)); // VK_F23
        key_map.insert("f24".into(), (0x87, false)); // VK_F24
        key_map.insert("numlock".into(), (0x90, false)); // VK_NUMLOCK
        key_map.insert("scrolllock".into(), (0x91, false)); // VK_SCROLL
        key_map.insert("shift_l".into(), (0xa0, false)); // VK_LSHIFT
        key_map.insert("shift".into(), (0xa0, false)); // VK_LSHIFT
        key_map.insert("shift_r".into(), (0xa1, false)); // VK_RSHIFT
        key_map.insert("control_l".into(), (0xa2, false)); // VK_LCONTROL
        key_map.insert("control".into(), (0xa2, false)); // VK_LCONTROL
        key_map.insert("ctrl".into(), (0xa2, false)); // VK_LCONTROL
        key_map.insert("control_r".into(), (0xa3, false)); // VK_RCONTROL
        key_map.insert("alt_l".into(), (0xa4, false)); // VK_LMENU
        key_map.insert("alt".into(), (0xa4, false)); // VK_LMENU
        key_map.insert("alt_r".into(), (0xa5, false)); // VK_RMENU
        key_map.insert("browserback".into(), (0xa6, false)); // VK_BROWSER_BACK
        key_map.insert("browserforward".into(), (0xa7, false)); // VK_BROWSER_FORWARD
        key_map.insert("browserrefresh".into(), (0xa8, false)); // VK_BROWSER_REFRESH
        key_map.insert("browserstop".into(), (0xa9, false)); // VK_BROWSER_STOP
        key_map.insert("browsersearch".into(), (0xaa, false)); // VK_BROWSER_SEARCH
        key_map.insert("browserfavorites".into(), (0xab, false)); // VK_BROWSER_FAVORITES
        key_map.insert("browserhome".into(), (0xac, false)); // VK_BROWSER_HOME
        key_map.insert("volumemute".into(), (0xad, false)); // VK_VOLUME_MUTE
        key_map.insert("volumedown".into(), (0xae, false)); // VK_VOLUME_DOWN
        key_map.insert("volumeup".into(), (0xaf, false)); // VK_VOLUME_UP
        key_map.insert("nexttrack".into(), (0xb0, false)); // VK_MEDIA_NEXT_TRACK
        key_map.insert("prevtrack".into(), (0xb1, false)); // VK_MEDIA_PREV_TRACK
        key_map.insert("stop".into(), (0xb2, false)); // VK_MEDIA_STOP
        key_map.insert("playpause".into(), (0xb3, false)); // VK_MEDIA_PLAY_PAUSE
        key_map.insert("launchmail".into(), (0xb4, false)); // VK_LAUNCH_MAIL
        key_map.insert("launchmediaselect".into(), (0xb5, false)); // VK_LAUNCH_MEDIA_SELECT
        key_map.insert("launchapp1".into(), (0xb6, false)); // VK_LAUNCH_APP1
        key_map.insert("launchapp2".into(), (0xb7, false)); // VK_LAUNCH_APP2
        key_map.insert("a".into(), (0x41, false)); // A
        key_map.insert("b".into(), (0x42, false)); // B
        key_map.insert("c".into(), (0x43, false)); // C
        key_map.insert("d".into(), (0x44, false)); // D
        key_map.insert("e".into(), (0x45, false)); // E
        key_map.insert("f".into(), (0x46, false)); // F
        key_map.insert("g".into(), (0x47, false)); // G
        key_map.insert("h".into(), (0x48, false)); // H
        key_map.insert("i".into(), (0x49, false)); // I
        key_map.insert("j".into(), (0x4A, false)); // J
        key_map.insert("k".into(), (0x4B, false)); // K
        key_map.insert("l".into(), (0x4C, false)); // L
        key_map.insert("m".into(), (0x4D, false)); // M
        key_map.insert("n".into(), (0x4E, false)); // N
        key_map.insert("o".into(), (0x4F, false)); // O
        key_map.insert("p".into(), (0x50, false)); // P
        key_map.insert("q".into(), (0x51, false)); // Q
        key_map.insert("r".into(), (0x52, false)); // R
        key_map.insert("s".into(), (0x53, false)); // S
        key_map.insert("t".into(), (0x54, false)); // T
        key_map.insert("u".into(), (0x55, false)); // U
        key_map.insert("v".into(), (0x56, false)); // V
        key_map.insert("w".into(), (0x57, false)); // W
        key_map.insert("x".into(), (0x58, false)); // X
        key_map.insert("y".into(), (0x59, false)); // Y
        key_map.insert("z".into(), (0x5A, false)); // Z
        key_map.insert("A".into(), (0x41, true)); // A
        key_map.insert("B".into(), (0x42, true)); // B
        key_map.insert("C".into(), (0x43, true)); // C
        key_map.insert("D".into(), (0x44, true)); // D
        key_map.insert("E".into(), (0x45, true)); // E
        key_map.insert("F".into(), (0x46, true)); // F
        key_map.insert("G".into(), (0x47, true)); // G
        key_map.insert("H".into(), (0x48, true)); // H
        key_map.insert("I".into(), (0x49, true)); // I
        key_map.insert("J".into(), (0x4A, true)); // J
        key_map.insert("K".into(), (0x4B, true)); // K
        key_map.insert("L".into(), (0x4C, true)); // L
        key_map.insert("M".into(), (0x4D, true)); // M
        key_map.insert("N".into(), (0x4E, true)); // N
        key_map.insert("O".into(), (0x4F, true)); // O
        key_map.insert("P".into(), (0x50, true)); // P
        key_map.insert("Q".into(), (0x51, true)); // Q
        key_map.insert("R".into(), (0x52, true)); // R
        key_map.insert("S".into(), (0x53, true)); // S
        key_map.insert("T".into(), (0x54, true)); // T
        key_map.insert("U".into(), (0x55, true)); // U
        key_map.insert("V".into(), (0x56, true)); // V
        key_map.insert("W".into(), (0x57, true)); // W
        key_map.insert("X".into(), (0x58, true)); // X
        key_map.insert("Y".into(), (0x59, true)); // Y
        key_map.insert("Z".into(), (0x5A, true)); // Z
        key_map.insert("0".into(), (0x30, false)); // 0
        key_map.insert("1".into(), (0x31, false)); // 1
        key_map.insert("2".into(), (0x32, false)); // 2
        key_map.insert("3".into(), (0x33, false)); // 3
        key_map.insert("4".into(), (0x34, false)); // 4
        key_map.insert("5".into(), (0x35, false)); // 5
        key_map.insert("6".into(), (0x36, false)); // 6
        key_map.insert("7".into(), (0x37, false)); // 7
        key_map.insert("8".into(), (0x38, false)); // 8
        key_map.insert("9".into(), (0x39, false)); // 9
        key_map.insert("0".into(), (0x30, false)); // 0
        key_map.insert("!".into(), (0x31, true)); // 1
        key_map.insert("@".into(), (0x32, true)); // 2
        key_map.insert("#".into(), (0x33, true)); // 3
        key_map.insert("$".into(), (0x34, true)); // 4
        key_map.insert("%".into(), (0x35, true)); // 5
        key_map.insert("^".into(), (0x36, true)); // 6
        key_map.insert("&".into(), (0x37, true)); // 7
        key_map.insert("*".into(), (0x38, true)); // 8
        key_map.insert("(".into(), (0x39, true)); // 9
        key_map.insert(".into()".into(), (0x30, true)); // 0
        key_map.insert(";".into(), (0xBA, false)); // VK_OEM_1
        key_map.insert(":".into(), (0xBA, true)); // VK_OEM_1
        key_map.insert("[".into(), (0xDB, false)); // [ { VK_OEM_4
        key_map.insert("]".into(), (0xDD, false)); // ] } VK_OEM_6
        key_map.insert("\\".into(), (0xDC, false)); // \ | VK_OEM_5
        key_map.insert("'".into(), (0xDE, false)); // ' " VK_OEM_7
        key_map.insert("{".into(), (0xDB, true)); // [ { VK_OEM_4
        key_map.insert("}".into(), (0xDD, true)); // ] } VK_OEM_6
        key_map.insert("|".into(), (0xDC, true)); // \ | VK_OEM_5
        key_map.insert("\"".into(), (0xDE, true)); // ' " VK_OEM_7
        key_map
    }
}
