#![allow(clippy::unit_arg)]

use crate::core::mouse::{mouse_position, Mouse, MouseClick, MouseScroll};
use crate::AutoGuiError;

impl crate::RustAutoGui {
    pub fn get_mouse_position(&self) -> Result<(i32, i32), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.get_mouse_position();
        #[cfg(target_os = "windows")]
        return Ok(Mouse::get_mouse_position());
        #[cfg(target_os = "macos")]
        return Mouse::get_mouse_position();
    }

    /// Move mouse to x,y pixel coordinate
    pub fn move_mouse_to_pos(&self, x: u32, y: u32, moving_time: f32) -> Result<(), AutoGuiError> {
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err(AutoGuiError::OutOfBoundsError(format!(
                "Out of bounds at positions x,y :{x}, {y}"
            )));
        }

        #[cfg(target_os = "windows")]
        {
            Mouse::move_mouse_to_pos(x as i32, y as i32, moving_time);
            Ok(())
        }
        #[cfg(target_os = "linux")]
        return self
            .mouse
            .move_mouse_to_pos(x as i32, y as i32, moving_time);
        #[cfg(target_os = "macos")]
        return Mouse::move_mouse_to_pos(x as i32, y as i32, moving_time);
    }

    /// Very similar to move mouse to pos, but takes Option<x> and Option<y>, where None value just keeps the current mouse x or y value
    /// So in case you want to more easily move mouse horizontally or vertically
    pub fn move_mouse_to(
        &self,
        x: Option<u32>,
        y: Option<u32>,
        moving_time: f32,
    ) -> Result<(), AutoGuiError> {
        let (pos_x, pos_y) = self.get_mouse_position()?;

        let x = if let Some(x) = x { x as i32 } else { pos_x };

        let y = if let Some(y) = y { y as i32 } else { pos_y };

        if (x > self.screen.screen_width) | (y > self.screen.screen_height) {
            return Err(AutoGuiError::OutOfBoundsError(format!(
                "Out of bounds at positions x,y :{x}, {y}"
            )));
        }

        #[cfg(target_os = "windows")]
        {
            Mouse::move_mouse_to_pos(x, y, moving_time);
            Ok(())
        }
        #[cfg(target_os = "linux")]
        return self.mouse.move_mouse_to_pos(x, y, moving_time);
        #[cfg(target_os = "macos")]
        return Mouse::move_mouse_to_pos(x, y, moving_time);
    }

    /// Move mouse in relative position. Accepts both positive and negative values, where negative X moves left, positive moves right
    /// and negative Y moves up, positive down
    pub fn move_mouse(&self, x: i32, y: i32, moving_time: f32) -> Result<(), AutoGuiError> {
        let (pos_x, pos_y) = self.get_mouse_position()?;

        let x = x + pos_x;
        let y = y + pos_y;

        if (x > self.screen.screen_width) | (y > self.screen.screen_height) | (x < 0) | (y < 0) {
            return Err(AutoGuiError::OutOfBoundsError(
                format!("Out of bounds at positions x,y :{x}, {y}"), // "Mouse movement out of screen boundaries".to_string(),
            ));
        }

        #[cfg(target_os = "windows")]
        {
            Mouse::move_mouse_to_pos(x, y, moving_time);
            Ok(())
        }
        #[cfg(target_os = "linux")]
        return self.mouse.move_mouse_to_pos(x, y, moving_time);
        #[cfg(target_os = "macos")]
        return Mouse::move_mouse_to_pos(x, y, moving_time);
    }

    /// executes left click down, move to position relative to current position, left click up
    pub fn drag_mouse(&self, x: i32, y: i32, moving_time: f32) -> Result<(), AutoGuiError> {
        let (pos_x, pos_y) = self.get_mouse_position()?;

        let x = x + pos_x;
        let y = y + pos_y;
        if (x > self.screen.screen_width) | (y > self.screen.screen_height) | (x < 0) | (y < 0) {
            return Err(AutoGuiError::OutOfBoundsError(
                format!("Out of bounds at positions x,y :{x}, {y}"), // "Mouse movement out of screen boundaries".to_string(),
            ));
        };
        #[cfg(target_os = "windows")]
        {
            Mouse::drag_mouse(x, y, moving_time);

            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
            return Mouse::drag_mouse(x as i32, y as i32, moving_time);
        }
        #[cfg(target_os = "linux")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
            self.mouse.drag_mouse(x as i32, y as i32, moving_time)
        }
    }

    /// Moves to position x,y. None values maintain current position. Useful for vertical and horizontal movement
    pub fn drag_mouse_to(
        &self,
        x: Option<u32>,
        y: Option<u32>,
        moving_time: f32,
    ) -> Result<(), AutoGuiError> {
        let (pos_x, pos_y) = self.get_mouse_position()?;

        let x = if let Some(x) = x { x as i32 } else { pos_x };

        let y = if let Some(y) = y { y as i32 } else { pos_y };

        if (x > self.screen.screen_width) | (y > self.screen.screen_height) {
            return Err(AutoGuiError::OutOfBoundsError(format!(
                "Out of bounds at positions x,y :{x}, {y}"
            )));
        }
        #[cfg(target_os = "windows")]
        {
            Mouse::drag_mouse(x, y, moving_time);

            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
            return Mouse::drag_mouse(x as i32, y as i32, moving_time);
        }
        #[cfg(target_os = "linux")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
            self.mouse.drag_mouse(x as i32, y as i32, moving_time)
        }
    }

    /// moves mouse to x, y pixel coordinate
    pub fn drag_mouse_to_pos(&self, x: u32, y: u32, moving_time: f32) -> Result<(), AutoGuiError> {
        if (x as i32 > self.screen.screen_width) | (y as i32 > self.screen.screen_height) {
            return Err(AutoGuiError::OutOfBoundsError(
                "Drag Mouse out of screen boundaries".to_string(),
            ));
        }

        #[cfg(target_os = "windows")]
        {
            Mouse::drag_mouse(x as i32, y as i32, moving_time);

            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }
            return Mouse::drag_mouse(x as i32, y as i32, moving_time);
        }
        #[cfg(target_os = "linux")]
        {
            if moving_time < 0.5 && !self.suppress_warnings {
                eprintln!("WARNING:Small moving time values may cause issues on mouse drag");
            }

            self.mouse.drag_mouse(x as i32, y as i32, moving_time)
        }
    }

    /// Mouse click. Choose button Mouseclick::{LEFT,RIGHT,MIDDLE}
    pub fn click(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_click(button);
        #[cfg(target_os = "windows")]
        return Ok(Mouse::mouse_click(button));
        #[cfg(target_os = "macos")]
        return Mouse::mouse_click(button);
    }

    /// executes left mouse click
    pub fn left_click(&self) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_click(MouseClick::LEFT);
        #[cfg(target_os = "windows")]
        return Ok(Mouse::mouse_click(MouseClick::LEFT));
        #[cfg(target_os = "macos")]
        return Mouse::mouse_click(MouseClick::LEFT);
    }

    /// executes right mouse click
    pub fn right_click(&self) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_click(MouseClick::RIGHT);
        #[cfg(target_os = "macos")]
        return Mouse::mouse_click(MouseClick::RIGHT);
        #[cfg(target_os = "windows")]
        return Ok(Mouse::mouse_click(MouseClick::RIGHT));
    }

    /// executes middle mouse click
    pub fn middle_click(&self) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_click(MouseClick::MIDDLE);
        #[cfg(target_os = "windows")]
        return Ok(Mouse::mouse_click(MouseClick::MIDDLE));
        #[cfg(target_os = "macos")]
        return Mouse::mouse_click(MouseClick::MIDDLE);
    }

    /// executes double left mouse click
    pub fn double_click(&self) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        {
            self.mouse.mouse_click(MouseClick::LEFT)?;
            self.mouse.mouse_click(MouseClick::LEFT)
        }
        #[cfg(target_os = "windows")]
        {
            Mouse::mouse_click(MouseClick::LEFT);
            Mouse::mouse_click(MouseClick::LEFT);
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Mouse::double_click()
    }

    pub fn click_down(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_down(button);
        #[cfg(target_os = "macos")]
        return Mouse::mouse_down(button);
        #[cfg(target_os = "windows")]
        return Ok(Mouse::mouse_down(button));
    }
    pub fn click_up(&self, button: MouseClick) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return self.mouse.mouse_up(button);
        #[cfg(target_os = "macos")]
        return Mouse::mouse_up(button);
        #[cfg(target_os = "windows")]
        return Ok(Mouse::mouse_up(button));
    }

    pub fn scroll_up(&self, intensity: u32) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return Ok(self.mouse.scroll(MouseScroll::UP, intensity));
        #[cfg(target_os = "windows")]
        return Ok(Mouse::scroll(MouseScroll::UP, intensity));
        #[cfg(target_os = "macos")]
        return Mouse::scroll(MouseScroll::UP, intensity);
    }

    pub fn scroll_down(&self, intensity: u32) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return Ok(self.mouse.scroll(MouseScroll::DOWN, intensity));
        #[cfg(target_os = "windows")]
        return Ok(Mouse::scroll(MouseScroll::DOWN, intensity));
        #[cfg(target_os = "macos")]
        return Mouse::scroll(MouseScroll::DOWN, intensity);
    }

    pub fn scroll_left(&self, intensity: u32) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return Ok(self.mouse.scroll(MouseScroll::LEFT, intensity));
        #[cfg(target_os = "windows")]
        return Ok(Mouse::scroll(MouseScroll::LEFT, intensity));
        #[cfg(target_os = "macos")]
        return Mouse::scroll(MouseScroll::LEFT, intensity);
    }

    pub fn scroll_right(&self, intensity: u32) -> Result<(), AutoGuiError> {
        #[cfg(target_os = "linux")]
        return Ok(self.mouse.scroll(MouseScroll::RIGHT, intensity));
        #[cfg(target_os = "windows")]
        return Ok(Mouse::scroll(MouseScroll::RIGHT, intensity));
        #[cfg(target_os = "macos")]
        return Mouse::scroll(MouseScroll::RIGHT, intensity);
    }
}
