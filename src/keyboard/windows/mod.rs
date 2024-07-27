extern crate winapi;
use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP,VK_SHIFT, KEYEVENTF_SCANCODE, VK_MENU, VK_CONTROL};
use winapi::um::winuser::{MapVirtualKeyW, MAPVK_VK_TO_VSC};
use std::mem::size_of;
use std::collections::HashMap;
use std::time::Duration;
use std::thread::sleep;


/// main struct for interacting with keyboard. Keymap is generated upon intialization. 
pub struct Keyboard {
    keymap : HashMap<String, u16>
}
impl Keyboard {
    /// create new keyboard instance.
    pub fn new ()-> Keyboard {
        let keyset = Keyboard::create_keymap();
        Keyboard {keymap:keyset}
    }

    unsafe fn key_down(scan_code: u16) {
        let mut input: INPUT = std::mem::zeroed();
        input.type_ = INPUT_KEYBOARD;
        {
            let ki = input.u.ki_mut();
            if scan_code == VK_SHIFT as u16 || scan_code == VK_CONTROL as u16 || scan_code == VK_MENU as u16 {
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
            if scan_code == VK_SHIFT as u16 || scan_code == VK_CONTROL as u16 || scan_code == VK_MENU as u16 {
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
            Keyboard::key_down(0x10);
            sleep(Duration::from_micros(50));
            // send key
            Keyboard::send_key(scan_code);
            sleep(Duration::from_micros(50));
            // release shift
            Keyboard::key_up(0x10);
            sleep(Duration::from_micros(50));
        }
    }
    

    /// function used when sending input as string
    pub fn send_char(&self, key:&char, shifted:&bool) {
        let char_string = String::from(*key);
        let value = self.keymap.get(&char_string);
        match value {
            Some(_)=> (),
            None => return,
        }
        let value = value.unwrap();
        if *shifted {
            Keyboard::send_shifted_key(*value);    
        } else {
            Keyboard::send_key(*value);
        }
    }

    /// function used when sending commands like "return" or "escape"
    pub fn send_command(&self, key:&String) {
        let value = self.keymap.get(key);
        let value = value.expect("Unknown command");
        Keyboard::send_key(*value);

    }


    pub fn send_multi_key(&self, key_1:&String, key_2:&String, key_3:Option<String>) {
        let value1 = self.keymap.get(key_1).expect("Invalid first key argument");
        let value2 = self.keymap.get(key_2).expect("Invalid second key argument");
        
        let mut third_key = false;
        let value3 = match key_3 {
            Some(value) => {
                third_key = true;
                let value3 = self.keymap.get(&value).expect("Invalid third key argument");
                value3
            },
            None => {
                &0
            }   
        };
        unsafe {
            Keyboard::key_down(*value1);
            Keyboard::key_down(*value2);
            if third_key {
                Keyboard::key_down(*value3);
                Keyboard::key_up(*value3);
            }
            Keyboard::key_up(*value2);
            Keyboard::key_up(*value1);
        }
        
        


    }

    /// mapping made so  bigger variety of strings can be used when sending string as input. 
    /// for instance, instead of neccessity of sending "period", we can send ".". This means when sending a 
    /// string like url test.hr we dont need to send test, then send period, then send hr 
    pub fn create_keymap() -> HashMap<String, u16> {
        let mut key_map: HashMap<String, u16> = HashMap::new(); 
        // Inserting key mappings
        key_map.insert(String::from("backspace"), 0x08); // VK_BACK
        key_map.insert(String::from("b"), 0x08); // VK_BACK
        key_map.insert(String::from("super"), 0x5B); // VK_LWIN
        key_map.insert(String::from("tab"), 0x09); // VK_TAB
        key_map.insert(String::from("t"), 0x09); // VK_TAB
        key_map.insert(String::from("clear"), 0x0c); // VK_CLEAR
        key_map.insert(String::from("enter"), 0x0d); // VK_RETURN
        key_map.insert(String::from("n"), 0x0d); // VK_RETURN
        key_map.insert(String::from("return"), 0x0d); // VK_RETURN
        key_map.insert(String::from("shift_l"), 0x10); // VK_SHIFT
        key_map.insert(String::from("ctrl"), 0x11); // VK_CONTROL
        key_map.insert(String::from("alt"), 0x12); // VK_MENU
        key_map.insert(String::from("pause"), 0x13); // VK_PAUSE
        key_map.insert(String::from("caps_lock"), 0x14); // VK_CAPITAL
        key_map.insert(String::from("kana"), 0x15); // VK_KANA
        key_map.insert(String::from("hanguel"), 0x15); // VK_HANGUEL
        key_map.insert(String::from("hangul"), 0x15); // VK_HANGUL
        key_map.insert(String::from("junja"), 0x17); // VK_JUNJA
        key_map.insert(String::from("final"), 0x18); // VK_FINAL
        key_map.insert(String::from("hanja"), 0x19); // VK_HANJA
        key_map.insert(String::from("kanji"), 0x19); // VK_KANJI
        key_map.insert(String::from("esc"), 0x1b); // VK_ESCAPE
        key_map.insert(String::from("escape"), 0x1b); // VK_ESCAPE
        key_map.insert(String::from("convert"), 0x1c); // VK_CONVERT
        key_map.insert(String::from("nonconvert"), 0x1d); // VK_NONCONVERT
        key_map.insert(String::from("accept"), 0x1e); // VK_ACCEPT
        key_map.insert(String::from("modechange"), 0x1f); // VK_MODECHANGE
        key_map.insert(String::from(" "), 0x20); // VK_SPACE
        key_map.insert(String::from("space"), 0x20); // VK_SPACE
        key_map.insert(String::from("pgup"), 0x21); // VK_PRIOR
        key_map.insert(String::from("pgdn"), 0x22); // VK_NEXT
        key_map.insert(String::from("pageup"), 0x21); // VK_PRIOR
        key_map.insert(String::from("pagedown"), 0x22); // VK_NEXT
        key_map.insert(String::from("end"), 0x23); // VK_END
        key_map.insert(String::from("home"), 0x24); // VK_HOME
        key_map.insert(String::from("left"), 0x25); // VK_LEFT
        key_map.insert(String::from("up"), 0x26); // VK_UP
        key_map.insert(String::from("right"), 0x27); // VK_RIGHT
        key_map.insert(String::from("down"), 0x28); // VK_DOWN
        key_map.insert(String::from("select"), 0x29); // VK_SELECT
        key_map.insert(String::from("print"), 0x2a); // VK_PRINT
        key_map.insert(String::from("execute"), 0x2b); // VK_EXECUTE
        key_map.insert(String::from("prtsc"), 0x2c); // VK_SNAPSHOT
        key_map.insert(String::from("prtscr"), 0x2c); // VK_SNAPSHOT
        key_map.insert(String::from("prntscrn"), 0x2c); // VK_SNAPSHOT
        key_map.insert(String::from("printscreen"), 0x2c); // VK_SNAPSHOT
        key_map.insert(String::from("insert"), 0x2d); // VK_INSERT
        key_map.insert(String::from("del"), 0x2e); // VK_DELETE
        key_map.insert(String::from("delete"), 0x2e); // VK_DELETE
        key_map.insert(String::from("help"), 0x2f); // VK_HELP
        key_map.insert(String::from("win"), 0x5b); // VK_LWIN
        key_map.insert(String::from("winleft"), 0x5b); // VK_LWIN
        key_map.insert(String::from("winright"), 0x5c); // VK_RWIN
        key_map.insert(String::from("apps"), 0x5d); // VK_APPS
        key_map.insert(String::from("sleep"), 0x5f); // VK_SLEEP
        key_map.insert(String::from("num0"), 0x60); // VK_NUMPAD0
        key_map.insert(String::from("num1"), 0x61); // VK_NUMPAD1
        key_map.insert(String::from("num2"), 0x62); // VK_NUMPAD2
        key_map.insert(String::from("num3"), 0x63); // VK_NUMPAD3
        key_map.insert(String::from("num4"), 0x64); // VK_NUMPAD4
        key_map.insert(String::from("num5"), 0x65); // VK_NUMPAD5
        key_map.insert(String::from("num6"), 0x66); // VK_NUMPAD6
        key_map.insert(String::from("num7"), 0x67); // VK_NUMPAD7
        key_map.insert(String::from("num8"), 0x68); // VK_NUMPAD8
        key_map.insert(String::from("num9"), 0x69); // VK_NUMPAD9
        key_map.insert(String::from("*"), 0x6a); // VK_MULTIPLY
        key_map.insert(String::from("+"), 0x6b); // VK_ADD
        key_map.insert(String::from("separator"), 0x6c); // VK_SEPARATOR
        key_map.insert(String::from("-"), 0x6d); // VK_SUBTRACT
        key_map.insert(String::from("."), 0xBE); // VK_OEM_PERIOD
        key_map.insert(String::from(","), 0xBC); // VK_OEM_COMMA
        key_map.insert(String::from("/"), 0x6f); // VK_DIVIDE
        key_map.insert(String::from("f1"), 0x70); // VK_F1
        key_map.insert(String::from("f2"), 0x71); // VK_F2
        key_map.insert(String::from("f3"), 0x72); // VK_F3
        key_map.insert(String::from("f4"), 0x73); // VK_F4
        key_map.insert(String::from("f5"), 0x74); // VK_F5
        key_map.insert(String::from("f6"), 0x75); // VK_F6
        key_map.insert(String::from("f7"), 0x76); // VK_F7
        key_map.insert(String::from("f8"), 0x77); // VK_F8
        key_map.insert(String::from("f9"), 0x78); // VK_F9
        key_map.insert(String::from("f10"), 0x79); // VK_F10
        key_map.insert(String::from("f11"), 0x7a); // VK_F11
        key_map.insert(String::from("f12"), 0x7b); // VK_F12
        key_map.insert(String::from("f13"), 0x7c); // VK_F13
        key_map.insert(String::from("f14"), 0x7d); // VK_F14
        key_map.insert(String::from("f15"), 0x7e); // VK_F15
        key_map.insert(String::from("f16"), 0x7f); // VK_F16
        key_map.insert(String::from("f17"), 0x80); // VK_F17
        key_map.insert(String::from("f18"), 0x81); // VK_F18
        key_map.insert(String::from("f19"), 0x82); // VK_F19
        key_map.insert(String::from("f20"), 0x83); // VK_F20
        key_map.insert(String::from("f21"), 0x84); // VK_F21
        key_map.insert(String::from("f22"), 0x85); // VK_F22
        key_map.insert(String::from("f23"), 0x86); // VK_F23
        key_map.insert(String::from("f24"), 0x87); // VK_F24
        key_map.insert(String::from("numlock"), 0x90); // VK_NUMLOCK
        key_map.insert(String::from("scrolllock"), 0x91); // VK_SCROLL
        key_map.insert(String::from("shiftleft"), 0xa0); // VK_LSHIFT
        key_map.insert(String::from("shiftright"), 0xa1); // VK_RSHIFT
        key_map.insert(String::from("control_l"), 0xa2); // VK_LCONTROL
        key_map.insert(String::from("control_r"), 0xa3); // VK_RCONTROL
        key_map.insert(String::from("alt_l"), 0xa4); // VK_LMENU
        key_map.insert(String::from("alt_r"), 0xa5); // VK_RMENU
        key_map.insert(String::from("browserback"), 0xa6); // VK_BROWSER_BACK
        key_map.insert(String::from("browserforward"), 0xa7); // VK_BROWSER_FORWARD
        key_map.insert(String::from("browserrefresh"), 0xa8); // VK_BROWSER_REFRESH
        key_map.insert(String::from("browserstop"), 0xa9); // VK_BROWSER_STOP
        key_map.insert(String::from("browsersearch"), 0xaa); // VK_BROWSER_SEARCH
        key_map.insert(String::from("browserfavorites"), 0xab); // VK_BROWSER_FAVORITES
        key_map.insert(String::from("browserhome"), 0xac); // VK_BROWSER_HOME
        key_map.insert(String::from("volumemute"), 0xad); // VK_VOLUME_MUTE
        key_map.insert(String::from("volumedown"), 0xae); // VK_VOLUME_DOWN
        key_map.insert(String::from("volumeup"), 0xaf); // VK_VOLUME_UP
        key_map.insert(String::from("nexttrack"), 0xb0); // VK_MEDIA_NEXT_TRACK
        key_map.insert(String::from("prevtrack"), 0xb1); // VK_MEDIA_PREV_TRACK
        key_map.insert(String::from("stop"), 0xb2); // VK_MEDIA_STOP
        key_map.insert(String::from("playpause"), 0xb3); // VK_MEDIA_PLAY_PAUSE
        key_map.insert(String::from("launchmail"), 0xb4); // VK_LAUNCH_MAIL
        key_map.insert(String::from("launchmediaselect"), 0xb5); // VK_LAUNCH_MEDIA_SELECT
        key_map.insert(String::from("launchapp1"), 0xb6); // VK_LAUNCH_APP1
        key_map.insert(String::from("launchapp2"), 0xb7); // VK_LAUNCH_APP2
        key_map.insert(String::from("a"), 0x41); // A
        key_map.insert(String::from("b"), 0x42); // B
        key_map.insert(String::from("c"), 0x43); // C
        key_map.insert(String::from("d"), 0x44); // D
        key_map.insert(String::from("e"), 0x45); // E
        key_map.insert(String::from("f"), 0x46); // F
        key_map.insert(String::from("g"), 0x47); // G
        key_map.insert(String::from("h"), 0x48); // H
        key_map.insert(String::from("i"), 0x49); // I
        key_map.insert(String::from("j"), 0x4A); // J
        key_map.insert(String::from("k"), 0x4B); // K
        key_map.insert(String::from("l"), 0x4C); // L
        key_map.insert(String::from("m"), 0x4D); // M
        key_map.insert(String::from("n"), 0x4E); // N
        key_map.insert(String::from("o"), 0x4F); // O
        key_map.insert(String::from("p"), 0x50); // P
        key_map.insert(String::from("q"), 0x51); // Q
        key_map.insert(String::from("r"), 0x52); // R
        key_map.insert(String::from("s"), 0x53); // S
        key_map.insert(String::from("t"), 0x54); // T
        key_map.insert(String::from("u"), 0x55); // U
        key_map.insert(String::from("v"), 0x56); // V
        key_map.insert(String::from("w"), 0x57); // W
        key_map.insert(String::from("x"), 0x58); // X
        key_map.insert(String::from("y"), 0x59); // Y
        key_map.insert(String::from("z"), 0x5A); // Z

        key_map.insert(String::from("0"), 0x30); // 0
        key_map.insert(String::from("1"), 0x31); // 1
        key_map.insert(String::from("2"), 0x32); // 2
        key_map.insert(String::from("3"), 0x33); // 3
        key_map.insert(String::from("4"), 0x34); // 4
        key_map.insert(String::from("5"), 0x35); // 5
        key_map.insert(String::from("6"), 0x36); // 6
        key_map.insert(String::from("7"), 0x37); // 7
        key_map.insert(String::from("8"), 0x38); // 8
        key_map.insert(String::from("9"), 0x39); // 9 
        key_map.insert(String::from("["), 0xDB); // [ { VK_OEM_4
        key_map.insert(String::from("]"), 0xDD); // ] } VK_OEM_6
        key_map.insert(String::from("\\"), 0xDC); // \ | VK_OEM_5
        key_map.insert(String::from("'"), 0xDE); // ' " VK_OEM_7
        key_map.insert(String::from("\\"), 0xDC); // \ | VK_OEM_5
        key_map
    }
}





    