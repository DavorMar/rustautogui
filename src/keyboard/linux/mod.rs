use std::{collections::HashMap, ffi::CString, process::Command, thread, time::Duration};
use x11::xlib::*;
use x11::xtest::*;

/// main struct for interacting with keyboard. Keymap is generated upon intialization.
/// screen is stored from Screen struct, where pointer for same screen object is used across the code
pub struct Keyboard {
    keymap: HashMap<String, (String, bool)>,
    screen: *mut _XDisplay,
}
impl Keyboard {
    /// create new keyboard instance. Display object is needed as argument
    pub fn new(screen: *mut _XDisplay) -> Self {
        // for future development
        let is_us_layout: bool = Self::is_us_layout();

        let keymap = Keyboard::create_keymap(is_us_layout);
        Self {
            keymap: keymap,
            screen: screen,
        }
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
                .trim()
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
    fn send_shifted_key(&self, scan_code: u32) -> Result<(), &'static str> {
        unsafe {
            let mut keysym_to_keycode2 = HashMap::new();
            let key_cstring = match CString::new("Shift_L".to_string()) {
                Ok(x) => x.as_ptr(),
                Err(_) => return Err("failed grabbing shift key"),
            };

            let keysym = XStringToKeysym(key_cstring);
            if !keysym_to_keycode2.contains_key(&keysym) {
                let keycode = XKeysymToKeycode(self.screen, keysym) as u32;
                keysym_to_keycode2.insert(keysym, keycode);
            }
            let keycode = keysym_to_keycode2[&keysym];
            self.press_key(keycode); //press shift
            self.send_key(scan_code);
            self.release_key(keycode); // release shift
        }
        Ok(())
    }

    /// grabs the value from structs keymap, then converts String to Keysim, and then keysim to Keycode.
    unsafe fn get_keycode(&self, key: &String) -> Result<(u32, bool), &'static str> {
        let value = self.keymap.get(key);

        let mut keysym_to_keycode = HashMap::new();
        let (keysym, shifted) = match value {
            Some(x) => {
                let shifted = x.1;
                let key_cstring = CString::new(x.0.clone());
                let key_cstring = key_cstring.map_err(|_| "failed to grab key value")?;
                let key_cstring = key_cstring.as_ptr();
                (XStringToKeysym(key_cstring), shifted)
            }
            None => return Err("failed to grab keystring"),
        };
        if keysym == 0 {
            return Err("failed to grab keystring");
        }
        if !keysym_to_keycode.contains_key(&keysym) {
            let keycode = XKeysymToKeycode(self.screen, keysym) as u32;
            keysym_to_keycode.insert(keysym, keycode);
        }
        let keycode = keysym_to_keycode[&keysym];
        Ok((keycode, shifted))
    }

    /// top level send character function that converts char to keycode and executes send key
    pub fn send_char(&self, key: &char) -> Result<(), &'static str> {
        unsafe {
            let char_string: String = String::from(*key);
            let (keycode, shifted) = self.get_keycode(&char_string)?;
            if keycode == 0 {
                return Err("couldnt input a key");
            }

            if shifted {
                self.send_shifted_key(keycode)?;
            } else {
                self.send_key(keycode);
            }
        }
        return Ok(());
    }

    /// similar to send char, but can be string such as return, escape etc
    pub fn send_command(&self, key: &String) -> Result<(), &'static str> {
        unsafe {
            let keycode = self.get_keycode(key)?;
            self.send_key(keycode.0);
        }
        return Ok(());
    }

    pub fn send_multi_key(
        &self,
        key_1: &String,
        key_2: &String,
        key_3: Option<String>,
    ) -> Result<(), &'static str> {
        unsafe {
            let value1 = self.get_keycode(&key_1)?;

            let value2 = self.get_keycode(&key_2)?;
            let mut third_key = false;
            let value3 = match key_3 {
                Some(value) => {
                    third_key = true;

                    let value3 = self.get_keycode(&value)?;
                    value3
                }
                None => (0, false), // this value should never be executed
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
    fn create_keymap(is_us_layout: bool) -> HashMap<String, (String, bool)> {
        let mut keysym_map: HashMap<String, (String, bool)> = HashMap::new();
        keysym_map.insert(
            String::from(String::from(" ")),
            (String::from("space"), false),
        );
        keysym_map.insert(String::from("!"), (String::from("exclam"), true));
        keysym_map.insert(String::from("\""), (String::from("quotedbl"), true));
        keysym_map.insert(String::from("#"), (String::from("numbersign"), true));
        keysym_map.insert(String::from("$"), (String::from("dollar"), true));
        keysym_map.insert(String::from("%"), (String::from("percent"), true));
        keysym_map.insert(String::from("&"), (String::from("ampersand"), true));
        keysym_map.insert(String::from("'"), (String::from("apostrophe"), false));
        keysym_map.insert(String::from("("), (String::from("parenleft"), false));
        keysym_map.insert(String::from(")"), (String::from("parenright"), false));
        keysym_map.insert(String::from("*"), (String::from("asterisk"), true));
        keysym_map.insert(String::from("+"), (String::from("plus"), true));
        keysym_map.insert(String::from(","), (String::from("comma"), false));
        keysym_map.insert(String::from("<"), (String::from("comma"), true));
        keysym_map.insert(String::from("-"), (String::from("minus"), false));
        keysym_map.insert(String::from("."), (String::from("period"), false));
        keysym_map.insert(String::from(">"), (String::from("period"), true));
        keysym_map.insert(String::from("/"), (String::from("slash"), false));
        keysym_map.insert(String::from("0"), (String::from("0"), false));
        keysym_map.insert(String::from("1"), (String::from("1"), false));
        keysym_map.insert(String::from("2"), (String::from("2"), false));
        keysym_map.insert(String::from("3"), (String::from("3"), false));
        keysym_map.insert(String::from("4"), (String::from("4"), false));
        keysym_map.insert(String::from("5"), (String::from("5"), false));
        keysym_map.insert(String::from("6"), (String::from("6"), false));
        keysym_map.insert(String::from("7"), (String::from("7"), false));
        keysym_map.insert(String::from("8"), (String::from("8"), false));
        keysym_map.insert(String::from("9"), (String::from("9"), false));
        keysym_map.insert(String::from(":"), (String::from("colon"), true));
        keysym_map.insert(String::from(";"), (String::from("semicolon"), false));
        keysym_map.insert(String::from("-"), (String::from("less"), false));
        keysym_map.insert(String::from("="), (String::from("equal"), false));
        keysym_map.insert(String::from("?"), (String::from("question"), true));
        keysym_map.insert(String::from("@"), (String::from("at"), true));
        keysym_map.insert(String::from("A"), (String::from("A"), true));
        keysym_map.insert(String::from("B"), (String::from("B"), true));
        keysym_map.insert(String::from("C"), (String::from("C"), true));
        keysym_map.insert(String::from("D"), (String::from("D"), true));
        keysym_map.insert(String::from("E"), (String::from("E"), true));
        keysym_map.insert(String::from("F"), (String::from("F"), true));
        keysym_map.insert(String::from("G"), (String::from("G"), true));
        keysym_map.insert(String::from("H"), (String::from("H"), true));
        keysym_map.insert(String::from("I"), (String::from("I"), true));
        keysym_map.insert(String::from("J"), (String::from("J"), true));
        keysym_map.insert(String::from("K"), (String::from("K"), true));
        keysym_map.insert(String::from("L"), (String::from("L"), true));
        keysym_map.insert(String::from("M"), (String::from("M"), true));
        keysym_map.insert(String::from("N"), (String::from("N"), true));
        keysym_map.insert(String::from("O"), (String::from("O"), true));
        keysym_map.insert(String::from("P"), (String::from("P"), true));
        keysym_map.insert(String::from("Q"), (String::from("Q"), true));
        keysym_map.insert(String::from("R"), (String::from("R"), true));
        keysym_map.insert(String::from("S"), (String::from("S"), true));
        keysym_map.insert(String::from("T"), (String::from("T"), true));
        keysym_map.insert(String::from("U"), (String::from("U"), true));
        keysym_map.insert(String::from("V"), (String::from("V"), true));
        keysym_map.insert(String::from("W"), (String::from("W"), true));
        keysym_map.insert(String::from("X"), (String::from("X"), true));
        keysym_map.insert(String::from("Y"), (String::from("Y"), true));
        keysym_map.insert(String::from("Z"), (String::from("Z"), true));
        keysym_map.insert(String::from("["), (String::from("bracketleft"), false));
        keysym_map.insert(String::from("\\"), (String::from("backslash"), false));
        keysym_map.insert(String::from("]"), (String::from("bracketright"), false));
        keysym_map.insert(String::from("_"), (String::from("underscore"), false));
        keysym_map.insert(String::from("a"), (String::from("a"), false));
        keysym_map.insert(String::from("b"), (String::from("b"), false));
        keysym_map.insert(String::from("c"), (String::from("c"), false));
        keysym_map.insert(String::from("d"), (String::from("d"), false));
        keysym_map.insert(String::from("e"), (String::from("e"), false));
        keysym_map.insert(String::from("f"), (String::from("f"), false));
        keysym_map.insert(String::from("g"), (String::from("g"), false));
        keysym_map.insert(String::from("h"), (String::from("h"), false));
        keysym_map.insert(String::from("i"), (String::from("i"), false));
        keysym_map.insert(String::from("j"), (String::from("j"), false));
        keysym_map.insert(String::from("k"), (String::from("k"), false));
        keysym_map.insert(String::from("l"), (String::from("l"), false));
        keysym_map.insert(String::from("m"), (String::from("m"), false));
        keysym_map.insert(String::from("n"), (String::from("n"), false));
        keysym_map.insert(String::from("o"), (String::from("o"), false));
        keysym_map.insert(String::from("p"), (String::from("p"), false));
        keysym_map.insert(String::from("q"), (String::from("q"), false));
        keysym_map.insert(String::from("r"), (String::from("r"), false));
        keysym_map.insert(String::from("s"), (String::from("s"), false));
        keysym_map.insert(String::from("t"), (String::from("t"), false));
        keysym_map.insert(String::from("u"), (String::from("u"), false));
        keysym_map.insert(String::from("v"), (String::from("v"), false));
        keysym_map.insert(String::from("w"), (String::from("w"), false));
        keysym_map.insert(String::from("x"), (String::from("x"), false));
        keysym_map.insert(String::from("y"), (String::from("y"), false));
        keysym_map.insert(String::from("z"), (String::from("z"), false));
        keysym_map.insert(String::from("{"), (String::from("braceleft"), true));
        keysym_map.insert(String::from("|"), (String::from("bar"), true));
        keysym_map.insert(String::from("}"), (String::from("braceright"), true));
        keysym_map.insert(String::from("~"), (String::from("asciitilde"), false));
        keysym_map.insert(String::from("shift_l"), (String::from("Shift_L"), false));
        keysym_map.insert(String::from("shift"), (String::from("Shift_L"), false));
        keysym_map.insert(String::from("shift_r"), (String::from("Shift_R"), false));
        keysym_map.insert(
            String::from("control_l"),
            (String::from("Control_L"), false),
        );
        keysym_map.insert(String::from("control"), (String::from("Control_L"), false));
        keysym_map.insert(String::from("ctrl"), (String::from("Control_L"), false));
        keysym_map.insert(
            String::from("control_r"),
            (String::from("Control_R"), false),
        );
        keysym_map.insert(
            String::from("caps_lock"),
            (String::from("Caps_Lock"), false),
        );
        keysym_map.insert(String::from("return"), (String::from("Return"), false));
        keysym_map.insert(String::from("enter"), (String::from("Return"), false));
        keysym_map.insert(
            String::from("backspace"),
            (String::from("BackSpace"), false),
        );
        keysym_map.insert(String::from("tab"), (String::from("Tab"), false));
        keysym_map.insert(String::from("escape"), (String::from("Escape"), false));
        keysym_map.insert(String::from("esc"), (String::from("Escape"), false));
        keysym_map.insert(String::from("delete"), (String::from("Delete"), false));
        keysym_map.insert(String::from("home"), (String::from("Home"), false));
        keysym_map.insert(String::from("left_arrow"), (String::from("Left"), false));
        keysym_map.insert(String::from("left"), (String::from("Left"), false));
        keysym_map.insert(String::from("up_arrow"), (String::from("Up"), false));
        keysym_map.insert(String::from("up"), (String::from("Up"), false));
        keysym_map.insert(String::from("right_arrow"), (String::from("Right"), false));
        keysym_map.insert(String::from("right"), (String::from("Right"), false));
        keysym_map.insert(String::from("down_arrow"), (String::from("Down"), false));
        keysym_map.insert(String::from("down"), (String::from("Down"), false));
        keysym_map.insert(String::from("end"), (String::from("End"), false));
        keysym_map.insert(String::from("alt_l"), (String::from("Alt_L"), false));
        keysym_map.insert(String::from("alt"), (String::from("Alt_L"), false));
        keysym_map.insert(String::from("alt_r"), (String::from("Alt_R"), false));
        // keysym_map.insert(String::from(" "), (String::from("Space"),false));
        keysym_map
    }
}
