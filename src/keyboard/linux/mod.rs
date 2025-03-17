use std::{collections::HashMap, ffi::CString, process::Command, thread, time::Duration};
use x11::xlib::*;
use x11::xtest::*;

/// main struct for interacting with keyboard. Keymap is generated upon intialization.
/// screen is stored from Screen struct, where pointer for same screen object is used across the code
pub struct Keyboard {
    keymap: HashMap<&'static str, (&'static str, bool)>,
    screen: *mut _XDisplay,
}
impl Keyboard {
    /// create new keyboard instance. Display object is needed as argument
    pub fn new(screen: *mut _XDisplay) -> Self {
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
    fn send_shifted_key(&self, scan_code: u32) -> Result<(), &'static str> {
        unsafe {
            let mut keysym_to_keycode2 = HashMap::new();
            let key_cstring = CString::new("Shift_L").map_err(|_| "failed grabbing shift key")?;

            let keysym = XStringToKeysym(key_cstring.as_ptr());
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

    unsafe fn get_keycode(&self, key: &str) -> Result<(u32, bool), &'static str> {
        let value = self.keymap.get(key);

        let mut keysym_to_keycode = HashMap::new();
        let (keysym, shifted) = match value {
            Some(x) => {
                let shifted = x.1;
                let key_cstring = CString::new(x.0).map_err(|_| "failed to grab key value")?;
                (XStringToKeysym(key_cstring.as_ptr()), shifted)
            }
            None => return Err("failed to grab keystring"),
        };
        if keysym == 0 {
            return Err("failed to grab keystring");
        }
        keysym_to_keycode
            .entry(keysym)
            .or_insert_with(|| XKeysymToKeycode(self.screen, keysym) as u32);
        let keycode = keysym_to_keycode[&keysym];
        Ok((keycode, shifted))
    }

    /// top level send character function that converts char to keycode and executes send key
    pub fn send_char(&self, key: char) -> Result<(), &'static str> {
        unsafe {
            let char_string: String = String::from(key);
            let keycode = self.get_keycode(&char_string)?;
            if keycode == (0, false) {
                return Err("couldnt input a key");
            }
            let shifted = keycode.1;
            let keycode = keycode.0;

            if shifted {
                self.send_shifted_key(keycode)?;
            } else {
                self.send_key(keycode);
            }
        }
        Ok(())
    }

    /// similar to send char, but can be string such as return, escape etc
    pub fn send_command(&self, key: &str) -> Result<(), &'static str> {
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
    ) -> Result<(), &'static str> {
        unsafe {
            let value1 = self.get_keycode(key_1)?;
            println!("got value 1");

            let value2 = self.get_keycode(key_2)?;
            println!("got value 2");
            let mut third_key = false;
            let value3 = match key_3 {
                Some(value) => {
                    third_key = true;

                    self.get_keycode(value)?
                }
                None => (0, false),
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
    fn create_keymap(_is_us_layout: bool) -> HashMap<&'static str, (&'static str, bool)> {
        let mut keysym_map = HashMap::with_capacity(128);
        keysym_map.insert(" ", ("space", false));
        keysym_map.insert("!", ("exclam", true));
        keysym_map.insert("\"", ("quotedbl", true));
        keysym_map.insert("#", ("numbersign", true));
        keysym_map.insert("$", ("dollar", true));
        keysym_map.insert("%", ("percent", true));
        keysym_map.insert("&", ("ampersand", true));
        keysym_map.insert("'", ("apostrophe", false));
        keysym_map.insert("(", ("parenleft", false));
        keysym_map.insert(")", ("parenright", false));
        keysym_map.insert("*", ("asterisk", true));
        keysym_map.insert("+", ("plus", true));
        keysym_map.insert(",", ("comma", false));
        keysym_map.insert("<", ("comma", true));
        keysym_map.insert("-", ("minus", false));
        keysym_map.insert(".", ("period", false));
        keysym_map.insert(">", ("period", true));
        keysym_map.insert("/", ("slash", false));
        keysym_map.insert("0", ("0", false));
        keysym_map.insert("1", ("1", false));
        keysym_map.insert("2", ("2", false));
        keysym_map.insert("3", ("3", false));
        keysym_map.insert("4", ("4", false));
        keysym_map.insert("5", ("5", false));
        keysym_map.insert("6", ("6", false));
        keysym_map.insert("7", ("7", false));
        keysym_map.insert("8", ("8", false));
        keysym_map.insert("9", ("9", false));
        keysym_map.insert(":", ("colon", true));
        keysym_map.insert(";", ("semicolon", false));
        keysym_map.insert("-", ("less", false));
        keysym_map.insert("=", ("equal", false));
        keysym_map.insert("?", ("question", true));
        keysym_map.insert("@", ("at", true));
        keysym_map.insert("A", ("A", true));
        keysym_map.insert("B", ("B", true));
        keysym_map.insert("C", ("C", true));
        keysym_map.insert("D", ("D", true));
        keysym_map.insert("E", ("E", true));
        keysym_map.insert("F", ("F", true));
        keysym_map.insert("G", ("G", true));
        keysym_map.insert("H", ("H", true));
        keysym_map.insert("I", ("I", true));
        keysym_map.insert("J", ("J", true));
        keysym_map.insert("K", ("K", true));
        keysym_map.insert("L", ("L", true));
        keysym_map.insert("M", ("M", true));
        keysym_map.insert("N", ("N", true));
        keysym_map.insert("O", ("O", true));
        keysym_map.insert("P", ("P", true));
        keysym_map.insert("Q", ("Q", true));
        keysym_map.insert("R", ("R", true));
        keysym_map.insert("S", ("S", true));
        keysym_map.insert("T", ("T", true));
        keysym_map.insert("U", ("U", true));
        keysym_map.insert("V", ("V", true));
        keysym_map.insert("W", ("W", true));
        keysym_map.insert("X", ("X", true));
        keysym_map.insert("Y", ("Y", true));
        keysym_map.insert("Z", ("Z", true));
        keysym_map.insert("[", ("bracketleft", false));
        keysym_map.insert("\\", ("backslash", false));
        keysym_map.insert("]", ("bracketright", false));
        keysym_map.insert("_", ("underscore", false));
        keysym_map.insert("a", ("a", false));
        keysym_map.insert("b", ("b", false));
        keysym_map.insert("c", ("c", false));
        keysym_map.insert("d", ("d", false));
        keysym_map.insert("e", ("e", false));
        keysym_map.insert("f", ("f", false));
        keysym_map.insert("g", ("g", false));
        keysym_map.insert("h", ("h", false));
        keysym_map.insert("i", ("i", false));
        keysym_map.insert("j", ("j", false));
        keysym_map.insert("k", ("k", false));
        keysym_map.insert("l", ("l", false));
        keysym_map.insert("m", ("m", false));
        keysym_map.insert("n", ("n", false));
        keysym_map.insert("o", ("o", false));
        keysym_map.insert("p", ("p", false));
        keysym_map.insert("q", ("q", false));
        keysym_map.insert("r", ("r", false));
        keysym_map.insert("s", ("s", false));
        keysym_map.insert("t", ("t", false));
        keysym_map.insert("u", ("u", false));
        keysym_map.insert("v", ("v", false));
        keysym_map.insert("w", ("w", false));
        keysym_map.insert("x", ("x", false));
        keysym_map.insert("y", ("y", false));
        keysym_map.insert("z", ("z", false));
        keysym_map.insert("{", ("braceleft", true));
        keysym_map.insert("|", ("bar", true));
        keysym_map.insert("}", ("braceright", true));
        keysym_map.insert("~", ("asciitilde", false));
        keysym_map.insert("shift_l", ("Shift_L", false));
        keysym_map.insert("shift", ("Shift_L", false));
        keysym_map.insert("shift_r", ("Shift_R", false));
        keysym_map.insert("control_l", ("Control_L", false));
        keysym_map.insert("control", ("Control_L", false));
        keysym_map.insert("ctrl", ("Control_L", false));
        keysym_map.insert("control_r", ("Control_R", false));
        keysym_map.insert("caps_lock", ("Caps_Lock", false));
        keysym_map.insert("return", ("Return", false));
        keysym_map.insert("enter", ("Return", false));
        keysym_map.insert("backspace", ("BackSpace", false));
        keysym_map.insert("tab", ("Tab", false));
        keysym_map.insert("escape", ("Escape", false));
        keysym_map.insert("esc", ("Escape", false));
        keysym_map.insert("delete", ("Delete", false));
        keysym_map.insert("home", ("Home", false));
        keysym_map.insert("left_arrow", ("Left", false));
        keysym_map.insert("left", ("Left", false));
        keysym_map.insert("up_arrow", ("Up", false));
        keysym_map.insert("up", ("Up", false));
        keysym_map.insert("right_arrow", ("Right", false));
        keysym_map.insert("right", ("Right", false));
        keysym_map.insert("down_arrow", ("Down", false));
        keysym_map.insert("down", ("Down", false));
        keysym_map.insert("end", ("End", false));
        keysym_map.insert("alt_l", ("Alt_L", false));
        keysym_map.insert("alt", ("Alt_L", false));
        keysym_map.insert("alt_r", ("Alt_R", false));
        // keysym_map.insert(" ", ("Space" ,false));
        keysym_map
    }
}
