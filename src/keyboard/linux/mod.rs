use super::Str;
use crate::errors::AutoGuiError;
use crate::keyboard::get_keymap_key;
use std::{collections::HashMap, ffi::CString, process::Command, thread, time::Duration};
use x11::xlib::{CurrentTime, XFlush, XKeysymToKeycode, XStringToKeysym, _XDisplay};
use x11::xtest::XTestFakeKeyEvent;

/// main struct for interacting with keyboard. Keymap is generated upon intialization.
/// screen is stored from Screen struct, where pointer for same screen object is used across the code
pub struct Keyboard {
    pub keymap: HashMap<Str, (Str, bool)>,
    screen: *mut _XDisplay,
}
impl Keyboard {
    /// create new keyboard instance. Display object is needed as argument
    pub fn new(screen: *mut _XDisplay) -> Self {
        // for future development
        let is_us_layout: bool = Self::is_us_layout();

        let keymap = Keyboard::create_keymap(is_us_layout);
        Self { keymap, screen }
    }

    /// Function that presses key down. When sending key, press key down and release key is executed
    unsafe fn press_key(&self, keycode: u32) {
        XTestFakeKeyEvent(self.screen, keycode, 1, CurrentTime);
        XFlush(self.screen);
    }
    /// Function that releases key up. When sending key, press key down and release key is executed
    unsafe fn release_key(&self, keycode: u32) {
        XTestFakeKeyEvent(self.screen, keycode, 0, CurrentTime);
        XFlush(self.screen);
    }

    /// send a key by press down and release up
    fn send_key(&self, scan_code: u32) {
        unsafe {
            self.press_key(scan_code);
            self.release_key(scan_code);
        }
    }

    // fn is_us_layout_old(screen:*mut _XDisplay) ->bool {
    //     unsafe {
    //         let mut major = 0;
    //         let mut minor = 0;
    //         let mut op = 0;
    //         let mut event = 0;
    //         let mut error = 0;
    //         if XkbQueryExtension(screen, &mut op, &mut event, &mut error, &mut major, &mut minor) == 0 {
    //             eprintln!("XKB extension is not available. Cannot detect keyboard layout, switching to default US. This may create issue if other layout is being used on OS");
    //             return true;
    //         }
    //         let mut state: XkbStateRec = std::mem::zeroed();
    //         XkbGetState(screen, 0x0100, &mut state); // Get current keyboard state
    //         let group = state.group; // Keyboard layout group (0 = US in most cases)

    //         group == 0 // If it's 0, it's likely US. Otherwise, another layout
    //     }
    // }

    // currently not developed further
    fn is_us_layout() -> bool {
        let output = Command::new("setxkbmap")
            .arg("-query")
            .output()
            .expect("Failed to execute setxkbmap");

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Find the line containing "layout:"
        if let Some(line) = output_str.lines().find(|line| line.starts_with("layout:")) {
            // Extract the layouts and split by comma
            let layouts: Vec<&str> = line
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .split(',')
                .collect();

            // List of US-style QWERTY layouts
            let us_layouts = [
                "us", "gb", "ca", "dk", "se", "no", "fi", "es", "pt", "it", "nl",
            ];

            // Check if any of the layouts in the list match US-style QWERTY
            return layouts.iter().any(|&layout| us_layouts.contains(&layout));
        }

        false
    }

    /// execute send_key function but press Shift key before, and release it after
    fn send_shifted_key(&self, scan_code: u32) -> Result<(), AutoGuiError> {
        unsafe {
            let mut keysym_to_keycode2 = HashMap::new();
            let key_cstring = CString::new("Shift_L".to_string())?;
            let key_cstring = key_cstring.as_ptr();

            let keysym = XStringToKeysym(key_cstring);
            keysym_to_keycode2
                .entry(keysym)
                .or_insert_with(|| XKeysymToKeycode(self.screen, keysym) as u32);
            let keycode = keysym_to_keycode2[&keysym];
            self.press_key(keycode); //press shift
            self.send_key(scan_code);
            self.release_key(keycode); // release shift
        }
        Ok(())
    }

    /// grabs the value from structs keymap, then converts String to Keysim, and then keysim to Keycode.
    unsafe fn get_keycode(&self, key: &str) -> Result<(u32, bool), AutoGuiError> {
        let (value, shifted) = get_keymap_key(self, key)?;

        let mut keysym_to_keycode = HashMap::new();
        let key_cstring = CString::new(value.to_string())?;
        let key_cstring = key_cstring.as_ptr();

        let keysym = XStringToKeysym(key_cstring);

        if keysym == 0 {
            return Err(AutoGuiError::OSFailure(
                "Failed to convert xstring to keysym. Keysym received is 0".to_string(),
            ));
        }
        keysym_to_keycode
            .entry(keysym)
            .or_insert_with(|| XKeysymToKeycode(self.screen, keysym) as u32);
        let keycode = keysym_to_keycode[&keysym];
        if keycode == 0 {
            return Err(AutoGuiError::OSFailure(
                "Failed to convert keysym to keycode. Keycode received is 0".to_string(),
            ));
        }
        Ok((keycode, shifted))
    }

    /// top level send character function that converts char to keycode and executes send key
    pub fn send_char(&self, key: char) -> Result<(), AutoGuiError> {
        unsafe {
            let char_string: String = String::from(key);
            let (keycode, shifted) = self.get_keycode(&char_string)?;

            if shifted {
                self.send_shifted_key(keycode)?;
            } else {
                self.send_key(keycode);
            }
        }
        Ok(())
    }

    /// similar to send char, but can be string such as return, escape etc
    pub fn send_command(&self, key: &str) -> Result<(), AutoGuiError> {
        unsafe {
            let keycode = self.get_keycode(key)?;
            self.send_key(keycode.0);
        }
        Ok(())
    }

    pub fn send_multi_key(
        &self,
        key_1: &str,
        key_2: &str,
        key_3: Option<&str>,
    ) -> Result<(), AutoGuiError> {
        unsafe {
            let value1 = self.get_keycode(key_1)?;

            let value2 = self.get_keycode(key_2)?;
            let mut third_key = false;
            let value3 = match key_3 {
                Some(value) => {
                    third_key = true;

                    self.get_keycode(value)?
                }
                None => (0, false), // this value should never be pressed
            };

            self.press_key(value1.0);
            thread::sleep(Duration::from_millis(50));
            self.press_key(value2.0);
            if third_key {
                self.press_key(value3.0);
                self.release_key(value3.0);
            }
            self.release_key(value2.0);
            self.release_key(value1.0);
        }
        Ok(())
    }

    /// https://www.cl.cam.ac.uk/~mgk25/ucs/keysymdef.h
    /// mapping made so  bigger variety of strings can be used when sending string as input.
    /// for instance, instead of neccessity of sending "period", we can send ".". This means when sending a
    /// string like url test.hr we dont need to send test, then send period, then send hr
    #[allow(unused_variables)]
    fn create_keymap(is_us_layout: bool) -> HashMap<Str, (Str, bool)> {
        let mut keysym_map = HashMap::new();
        keysym_map.insert(" ".into(), ("space".into(), false));
        keysym_map.insert("!".into(), ("exclam".into(), true));
        keysym_map.insert("\"".into(), ("quotedbl".into(), true));
        keysym_map.insert("#".into(), ("numbersign".into(), true));
        keysym_map.insert("$".into(), ("dollar".into(), true));
        keysym_map.insert("%".into(), ("percent".into(), true));
        keysym_map.insert("&".into(), ("ampersand".into(), true));
        keysym_map.insert("'".into(), ("apostrophe".into(), false));
        keysym_map.insert("(".into(), ("parenleft".into(), false));
        keysym_map.insert(".into()".into(), ("parenright".into(), false));
        keysym_map.insert("*".into(), ("asterisk".into(), true));
        keysym_map.insert("+".into(), ("plus".into(), true));
        keysym_map.insert(",".into(), ("comma".into(), false));
        keysym_map.insert("<".into(), ("comma".into(), true));
        keysym_map.insert("-".into(), ("minus".into(), false));
        keysym_map.insert(".".into(), ("period".into(), false));
        keysym_map.insert(">".into(), ("period".into(), true));
        keysym_map.insert("/".into(), ("slash".into(), false));
        keysym_map.insert("0".into(), ("0".into(), false));
        keysym_map.insert("1".into(), ("1".into(), false));
        keysym_map.insert("2".into(), ("2".into(), false));
        keysym_map.insert("3".into(), ("3".into(), false));
        keysym_map.insert("4".into(), ("4".into(), false));
        keysym_map.insert("5".into(), ("5".into(), false));
        keysym_map.insert("6".into(), ("6".into(), false));
        keysym_map.insert("7".into(), ("7".into(), false));
        keysym_map.insert("8".into(), ("8".into(), false));
        keysym_map.insert("9".into(), ("9".into(), false));
        keysym_map.insert(":".into(), ("colon".into(), true));
        keysym_map.insert(";".into(), ("semicolon".into(), false));
        keysym_map.insert("-".into(), ("less".into(), false));
        keysym_map.insert("=".into(), ("equal".into(), false));
        keysym_map.insert("?".into(), ("question".into(), true));
        keysym_map.insert("@".into(), ("at".into(), true));
        keysym_map.insert("A".into(), ("A".into(), true));
        keysym_map.insert("B".into(), ("B".into(), true));
        keysym_map.insert("C".into(), ("C".into(), true));
        keysym_map.insert("D".into(), ("D".into(), true));
        keysym_map.insert("E".into(), ("E".into(), true));
        keysym_map.insert("F".into(), ("F".into(), true));
        keysym_map.insert("G".into(), ("G".into(), true));
        keysym_map.insert("H".into(), ("H".into(), true));
        keysym_map.insert("I".into(), ("I".into(), true));
        keysym_map.insert("J".into(), ("J".into(), true));
        keysym_map.insert("K".into(), ("K".into(), true));
        keysym_map.insert("L".into(), ("L".into(), true));
        keysym_map.insert("M".into(), ("M".into(), true));
        keysym_map.insert("N".into(), ("N".into(), true));
        keysym_map.insert("O".into(), ("O".into(), true));
        keysym_map.insert("P".into(), ("P".into(), true));
        keysym_map.insert("Q".into(), ("Q".into(), true));
        keysym_map.insert("R".into(), ("R".into(), true));
        keysym_map.insert("S".into(), ("S".into(), true));
        keysym_map.insert("T".into(), ("T".into(), true));
        keysym_map.insert("U".into(), ("U".into(), true));
        keysym_map.insert("V".into(), ("V".into(), true));
        keysym_map.insert("W".into(), ("W".into(), true));
        keysym_map.insert("X".into(), ("X".into(), true));
        keysym_map.insert("Y".into(), ("Y".into(), true));
        keysym_map.insert("Z".into(), ("Z".into(), true));
        keysym_map.insert("[".into(), ("bracketleft".into(), false));
        keysym_map.insert("\\".into(), ("backslash".into(), false));
        keysym_map.insert("]".into(), ("bracketright".into(), false));
        keysym_map.insert("_".into(), ("underscore".into(), false));
        keysym_map.insert("a".into(), ("a".into(), false));
        keysym_map.insert("b".into(), ("b".into(), false));
        keysym_map.insert("c".into(), ("c".into(), false));
        keysym_map.insert("d".into(), ("d".into(), false));
        keysym_map.insert("e".into(), ("e".into(), false));
        keysym_map.insert("f".into(), ("f".into(), false));
        keysym_map.insert("g".into(), ("g".into(), false));
        keysym_map.insert("h".into(), ("h".into(), false));
        keysym_map.insert("i".into(), ("i".into(), false));
        keysym_map.insert("j".into(), ("j".into(), false));
        keysym_map.insert("k".into(), ("k".into(), false));
        keysym_map.insert("l".into(), ("l".into(), false));
        keysym_map.insert("m".into(), ("m".into(), false));
        keysym_map.insert("n".into(), ("n".into(), false));
        keysym_map.insert("o".into(), ("o".into(), false));
        keysym_map.insert("p".into(), ("p".into(), false));
        keysym_map.insert("q".into(), ("q".into(), false));
        keysym_map.insert("r".into(), ("r".into(), false));
        keysym_map.insert("s".into(), ("s".into(), false));
        keysym_map.insert("t".into(), ("t".into(), false));
        keysym_map.insert("u".into(), ("u".into(), false));
        keysym_map.insert("v".into(), ("v".into(), false));
        keysym_map.insert("w".into(), ("w".into(), false));
        keysym_map.insert("x".into(), ("x".into(), false));
        keysym_map.insert("y".into(), ("y".into(), false));
        keysym_map.insert("z".into(), ("z".into(), false));
        keysym_map.insert("{".into(), ("braceleft".into(), true));
        keysym_map.insert("|".into(), ("bar".into(), true));
        keysym_map.insert("}".into(), ("braceright".into(), true));
        keysym_map.insert("~".into(), ("asciitilde".into(), false));
        keysym_map.insert("shift_l".into(), ("Shift_L".into(), false));
        keysym_map.insert("shift".into(), ("Shift_L".into(), false));
        keysym_map.insert("shift_r".into(), ("Shift_R".into(), false));
        keysym_map.insert("control_l".into(), ("Control_L".into(), false));
        keysym_map.insert("control".into(), ("Control_L".into(), false));
        keysym_map.insert("ctrl".into(), ("Control_L".into(), false));
        keysym_map.insert("control_r".into(), ("Control_R".into(), false));
        keysym_map.insert("caps_lock".into(), ("Caps_Lock".into(), false));
        keysym_map.insert("return".into(), ("Return".into(), false));
        keysym_map.insert("enter".into(), ("Return".into(), false));
        keysym_map.insert("backspace".into(), ("BackSpace".into(), false));
        keysym_map.insert("tab".into(), ("Tab".into(), false));
        keysym_map.insert("escape".into(), ("Escape".into(), false));
        keysym_map.insert("esc".into(), ("Escape".into(), false));
        keysym_map.insert("delete".into(), ("Delete".into(), false));
        keysym_map.insert("home".into(), ("Home".into(), false));
        keysym_map.insert("left_arrow".into(), ("Left".into(), false));
        keysym_map.insert("left".into(), ("Left".into(), false));
        keysym_map.insert("up_arrow".into(), ("Up".into(), false));
        keysym_map.insert("up".into(), ("Up".into(), false));
        keysym_map.insert("right_arrow".into(), ("Right".into(), false));
        keysym_map.insert("right".into(), ("Right".into(), false));
        keysym_map.insert("down_arrow".into(), ("Down".into(), false));
        keysym_map.insert("down".into(), ("Down".into(), false));
        keysym_map.insert("end".into(), ("End".into(), false));
        keysym_map.insert("alt_l".into(), ("Alt_L".into(), false));
        keysym_map.insert("alt".into(), ("Alt_L".into(), false));
        keysym_map.insert("alt_r".into(), ("Alt_R".into(), false));
        keysym_map.insert("win".into(), ("Super_L".into(), false));
        keysym_map.insert("win_l".into(), ("Super_L".into(), false));
        keysym_map.insert("winleft".into(), ("Super_L".into(), false));
        keysym_map.insert("super_l".into(), ("Super_L".into(), false));
        keysym_map.insert("win_r".into(), ("Super_R".into(), false));
        keysym_map.insert("winright".into(), ("Super_R".into(), false));
        keysym_map.insert("super_r".into(), ("Super_R".into(), false));

        keysym_map.insert("f1".into(), ("F1".into(), false));
        keysym_map.insert("f2".into(), ("F2".into(), false));
        keysym_map.insert("f3".into(), ("F3".into(), false));
        keysym_map.insert("f4".into(), ("F4".into(), false));
        keysym_map.insert("f5".into(), ("F5".into(), false));
        keysym_map.insert("f6".into(), ("F6".into(), false));
        keysym_map.insert("f7".into(), ("F7".into(), false));
        keysym_map.insert("f8".into(), ("F8".into(), false));
        keysym_map.insert("f9".into(), ("F9".into(), false));
        keysym_map.insert("f10".into(), ("F10".into(), false));
        keysym_map.insert("f11".into(), ("F11".into(), false));
        keysym_map.insert("f12".into(), ("F12".into(), false));
        keysym_map.insert("f13".into(), ("F13".into(), false));
        keysym_map.insert("f14".into(), ("F14".into(), false));
        keysym_map.insert("f15".into(), ("F15".into(), false));
        keysym_map.insert("f16".into(), ("F16".into(), false));
        keysym_map.insert("f17".into(), ("F17".into(), false));
        keysym_map.insert("f18".into(), ("F18".into(), false));
        keysym_map.insert("f19".into(), ("F19".into(), false));
        keysym_map.insert("f20".into(), ("F20".into(), false));

        // keysym_map.insert(" ".into(), ("Space".into(),false));
        keysym_map
    }
}
