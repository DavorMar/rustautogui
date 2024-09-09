use std::collections::HashMap;
use x11::xlib::*;
use x11::xtest::*;
use std::ffi::CString;

/// main struct for interacting with keyboard. Keymap is generated upon intialization. 
/// screen is stored from Screen struct, where pointer for same screen object is used across the code
pub struct Keyboard {
    keymap: HashMap<String,String>,
    screen : *mut _XDisplay,
}
impl Keyboard {
    /// create new keyboard instance. Display object is needed as argument
    pub fn new(screen: *mut _XDisplay) -> Self {
        let keymap = Keyboard::create_keymap();
        Self { keymap: keymap, screen:screen}
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


    /// execute send_key function but press Shift key before, and release it after
    fn send_shifted_key (&self, scan_code:u32) {
        unsafe {

            let mut keysym_to_keycode2 = HashMap::new();
            let keysym = XStringToKeysym(CString::new("Shift_L".to_string()).unwrap().as_ptr());
            if !keysym_to_keycode2.contains_key(&keysym) {
                let keycode = XKeysymToKeycode(self.screen, keysym) as u32;
                keysym_to_keycode2.insert(keysym, keycode);
            }
            let keycode = keysym_to_keycode2[&keysym];
            self.press_key(keycode); //press shift
            self.send_key(scan_code);
            self.release_key(keycode); // release shift
        }
    }

    unsafe fn get_keycode(&self, key: &String) -> u32 {
        let value = self.keymap.get(key);
        let mut keysym_to_keycode = HashMap::new();
        let keysym = match value {
            Some(x) => XStringToKeysym(CString::new(x.clone()).unwrap().as_ptr()),
            None => XStringToKeysym(CString::new(key.clone()).unwrap().as_ptr())
        };
        if keysym == 0 {
            
            return 0;
        }
        if !keysym_to_keycode.contains_key(&keysym) {
            let keycode = XKeysymToKeycode(self.screen, keysym) as u32;
            keysym_to_keycode.insert(keysym, keycode);
        }
        let keycode = keysym_to_keycode[&keysym];
        keycode
    }

    pub fn send_string(&self, string: &String) {
        string.chars().for_each(|c| {
            c.to_lowercase().for_each(|sub_c| {
                self.send_char(&sub_c, &c.is_uppercase())
            });
        });
    }
    
    /// top level send character function that converts char to keycode and executes send key
    pub fn send_char (&self, key:&char, shifted:&bool) {
        unsafe {
            let char_string: String = String::from(*key);
            let keycode = self.get_keycode(&char_string);
            if keycode == 0 {
                return
            }
            if *shifted {
                self.send_shifted_key(keycode);    
            } else {
                self.send_key(keycode);
            }
        }
        

        
    }

    /// similar to send char, but can be string such as return, escape etc
    pub fn send_command(&self, key:&String) {
        
        unsafe {
            let keycode = self.get_keycode(key);
            self.send_key(keycode);
        }
    }

    pub fn send_multi_key(&self, key_1:&String, key_2:&String, key_3:Option<String>) {

        unsafe {
            let value1 = self.keymap.get(key_1).expect("Invalid first key argument");
            let value1 = self.get_keycode(value1);

            let value2 = self.keymap.get(key_2).expect("Invalid second key argument");
            let value2 = self.get_keycode(value2);

            let mut third_key = false;
            let value3 = match key_3 {
                Some(value) => {
                    third_key = true;
                    let value3 = self.keymap.get(&value).expect("Invalid third key argument");
                    let value3 = self.get_keycode(value3);
                    value3
                },
                None => {
                    0
                }   
            };
        
            self.press_key(value1);
            self.press_key(value2);
            if third_key {
                self.press_key(value3);
                self.release_key(value3);
            }
            self.release_key(value2);
            self.release_key(value1);
        }

    }



    /// https://www.cl.cam.ac.uk/~mgk25/ucs/keysymdef.h
    /// mapping made so  bigger variety of strings can be used when sending string as input. 
    /// for instance, instead of neccessity of sending "period", we can send ".". This means when sending a 
    /// string like url test.hr we dont need to send test, then send period, then send hr 
    fn create_keymap () -> HashMap<String, String> {
        let mut keysym_map: HashMap<String, String> = HashMap::new();
        keysym_map.insert(String::from(String::from(" ")), String::from("space"));
        keysym_map.insert(String::from("!"), String::from("exclam"));
        keysym_map.insert(String::from("\""), String::from("quotedbl"));
        keysym_map.insert(String::from("#"), String::from("numbersign"));
        keysym_map.insert(String::from("$"), String::from("dollar"));
        keysym_map.insert(String::from("%"), String::from("percent"));
        keysym_map.insert(String::from("&"), String::from("ampersand"));
        keysym_map.insert(String::from("'"), String::from("apostrophe"));
        keysym_map.insert(String::from("("), String::from("parenleft"));
        keysym_map.insert(String::from(")"), String::from("parenright"));
        keysym_map.insert(String::from("*"), String::from("asterisk"));
        keysym_map.insert(String::from("+"), String::from("plus"));
        keysym_map.insert(String::from(","), String::from("comma"));
        keysym_map.insert(String::from("-"), String::from("minus"));
        keysym_map.insert(String::from("."), String::from("period"));
        keysym_map.insert(String::from("/"), String::from("slash"));
        keysym_map.insert(String::from("0"), String::from("0"));
        keysym_map.insert(String::from("1"), String::from("1"));
        keysym_map.insert(String::from("2"), String::from("2"));
        keysym_map.insert(String::from("3"), String::from("3"));
        keysym_map.insert(String::from("4"), String::from("4"));
        keysym_map.insert(String::from("5"), String::from("5"));
        keysym_map.insert(String::from("6"), String::from("6"));
        keysym_map.insert(String::from("7"), String::from("7"));
        keysym_map.insert(String::from("8"), String::from("8"));
        keysym_map.insert(String::from("9"), String::from("9"));
        keysym_map.insert(String::from(":"), String::from("colon"));
        keysym_map.insert(String::from(";"), String::from("semicolon"));
        keysym_map.insert(String::from("-"), String::from("less"));
        keysym_map.insert(String::from("="), String::from("equal"));
        keysym_map.insert(String::from(">"), String::from("greater"));
        keysym_map.insert(String::from("?"), String::from("question"));
        keysym_map.insert(String::from("@"), String::from("at"));
        keysym_map.insert(String::from("A"), String::from("A"));
        keysym_map.insert(String::from("B"), String::from("B"));
        keysym_map.insert(String::from("C"), String::from("C"));
        keysym_map.insert(String::from("D"), String::from("D"));
        keysym_map.insert(String::from("E"), String::from("E"));
        keysym_map.insert(String::from("F"), String::from("F"));
        keysym_map.insert(String::from("G"), String::from("G"));
        keysym_map.insert(String::from("H"), String::from("H"));
        keysym_map.insert(String::from("I"), String::from("I"));
        keysym_map.insert(String::from("J"), String::from("J"));
        keysym_map.insert(String::from("K"), String::from("K"));
        keysym_map.insert(String::from("L"), String::from("L"));
        keysym_map.insert(String::from("M"), String::from("M"));
        keysym_map.insert(String::from("N"), String::from("N"));
        keysym_map.insert(String::from("O"), String::from("O"));
        keysym_map.insert(String::from("P"), String::from("P"));
        keysym_map.insert(String::from("Q"), String::from("Q"));
        keysym_map.insert(String::from("R"), String::from("R"));
        keysym_map.insert(String::from("S"), String::from("S"));
        keysym_map.insert(String::from("T"), String::from("T"));
        keysym_map.insert(String::from("U"), String::from("U"));
        keysym_map.insert(String::from("V"), String::from("V"));
        keysym_map.insert(String::from("W"), String::from("W"));
        keysym_map.insert(String::from("X"), String::from("X"));
        keysym_map.insert(String::from("Y"), String::from("Y"));
        keysym_map.insert(String::from("Z"), String::from("Z"));
        keysym_map.insert(String::from("["), String::from("bracketleft"));
        keysym_map.insert(String::from("\\"), String::from("backslash"));
        keysym_map.insert(String::from("]"), String::from("bracketright"));
        keysym_map.insert(String::from("_"), String::from("underscore"));
        keysym_map.insert(String::from("a"), String::from("a"));
        keysym_map.insert(String::from("b"), String::from("b"));
        keysym_map.insert(String::from("c"), String::from("c"));
        keysym_map.insert(String::from("d"), String::from("d"));
        keysym_map.insert(String::from("e"), String::from("e"));
        keysym_map.insert(String::from("f"), String::from("f"));
        keysym_map.insert(String::from("g"), String::from("g"));
        keysym_map.insert(String::from("h"), String::from("h"));
        keysym_map.insert(String::from("i"), String::from("i"));
        keysym_map.insert(String::from("j"), String::from("j"));
        keysym_map.insert(String::from("k"), String::from("k"));
        keysym_map.insert(String::from("l"), String::from("l"));
        keysym_map.insert(String::from("m"), String::from("m"));
        keysym_map.insert(String::from("n"), String::from("n"));
        keysym_map.insert(String::from("o"), String::from("o"));
        keysym_map.insert(String::from("p"), String::from("p"));
        keysym_map.insert(String::from("q"), String::from("q"));
        keysym_map.insert(String::from("r"), String::from("r"));
        keysym_map.insert(String::from("s"), String::from("s"));
        keysym_map.insert(String::from("t"), String::from("t"));
        keysym_map.insert(String::from("u"), String::from("u"));
        keysym_map.insert(String::from("v"), String::from("v"));
        keysym_map.insert(String::from("w"), String::from("w"));
        keysym_map.insert(String::from("x"), String::from("x"));
        keysym_map.insert(String::from("y"), String::from("y"));
        keysym_map.insert(String::from("z"), String::from("z"));
        keysym_map.insert(String::from("{"), String::from("braceleft"));
        keysym_map.insert(String::from("|"), String::from("bar"));
        keysym_map.insert(String::from("}"), String::from("braceright"));
        keysym_map.insert(String::from("~"), String::from("asciitilde"));
        keysym_map.insert(String::from("shift_l"), String::from("Shift_L"));
        keysym_map.insert(String::from("shift_r"), String::from("Shift_R"));
        keysym_map.insert(String::from("control_l"), String::from("Control_L"));
        keysym_map.insert(String::from("control_r"), String::from("Control_R"));
        keysym_map.insert(String::from("caps_lock"), String::from("Caps_Lock"));
        keysym_map.insert(String::from("return"), String::from("Return"));
        keysym_map.insert(String::from("backspace"), String::from("BackSpace"));
        keysym_map.insert(String::from("tab"), String::from("Tab"));
        keysym_map.insert(String::from("delete"), String::from("Delete"));
        keysym_map.insert(String::from("home"), String::from("Home"));
        keysym_map.insert(String::from("left"), String::from("leftarrow"));
        keysym_map.insert(String::from("up"), String::from("uparrow"));
        keysym_map.insert(String::from("right"), String::from("rightarrow"));
        keysym_map.insert(String::from("down"), String::from("downarrow"));
        keysym_map.insert(String::from("end"), String::from("End"));
        keysym_map
    }
}