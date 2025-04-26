use crate::AutoGuiError;

impl crate::RustAutoGui {
    /// accepts string and mimics keyboard key presses for each character in string
    pub fn keyboard_input(&self, input: &str) -> Result<(), AutoGuiError> {
        let input_string = String::from(input);
        for letter in input_string.chars() {
            self.keyboard.send_char(&letter)?;
        }
        Ok(())
    }

    /// executes keyboard command like "return" or "escape"
    pub fn keyboard_command(&self, input: &str) -> Result<(), AutoGuiError> {
        let input_string = String::from(input);
        // return automatically the result of send_command function
        self.keyboard.send_command(&input_string)
    }

    pub fn keyboard_multi_key(
        &self,
        input1: &str,
        input2: &str,
        input3: Option<&str>,
    ) -> Result<(), AutoGuiError> {
        let input3 = match input3 {
            Some(x) => Some(String::from(x)),
            None => None,
        };
        // send automatically result of function
        self.keyboard.send_multi_key(input1, input2, input3)
    }

    pub fn key_down(&self, key: &str) -> Result<(), AutoGuiError> {
        self.keyboard.key_down(key)
    }

    pub fn key_up(&self, key: &str) -> Result<(), AutoGuiError> {
        self.keyboard.key_up(key)
    }
}
