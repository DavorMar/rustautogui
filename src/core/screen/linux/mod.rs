mod wayland;
mod x11;
use std::env;

#[cfg(not(feature = "lite"))]
use image::{GrayImage, ImageBuffer, Luma, Rgba, RgbaImage};
use wayland::WaylandScreen;
use x11::X11Screen;

use crate::core::CaptureableScreen;
#[cfg(not(feature = "lite"))]
use crate::errors::AutoGuiError;

pub enum Screen {
    Wayland(WaylandScreen),
    X11(X11Screen),
}

impl CaptureableScreen for Screen {
    fn new() -> Result<Self, AutoGuiError>
    where
        Self: std::marker::Sized,
    {
        let session = env::var("XDG_SESSION_TYPE").map_err(|_| {
            AutoGuiError::OSFailure(
                "XDG_SESSION_TYPE is not set, cannot determine wayland vs x11".to_string(),
            )
        })?;
        match session.as_str() {
            "wayland" => Ok(Screen::Wayland(WaylandScreen::new()?)),
            "x11" => Ok(Screen::X11(X11Screen::new()?)),
            unknown => Err(AutoGuiError::OSFailure(format!(
                "Unknown XDG_SESSION_TYPE={unknown} - should be 'x11' or 'wayland'"
            ))),
        }
    }

    /// returns screen dimensions. All monitors included
    fn dimension(&self) -> (i32, i32) {
        match self {
            Screen::Wayland(s) => s.dimension(),
            Screen::X11(s) => s.dimension(),
        }
    }

    fn destroy(&mut self) {
        match self {
            Screen::Wayland(s) => s.destroy(),
            Screen::X11(s) => s.destroy(),
        }
    }

    #[allow(dead_code)]
    /// executes convert_bitmap_to_rgba, meaning it converts Vector of values to RGBA and crops the image
    /// as inputted region area. Not used anywhere at the moment
    #[cfg(not(feature = "lite"))]
    fn grab_screen_image(
        &mut self,
        region: (u32, u32, u32, u32),
    ) -> Result<RgbaImage, AutoGuiError> {
        match self {
            Screen::Wayland(s) => s.grab_screen_image(region),
            Screen::X11(s) => s.grab_screen_image(region),
        }
    }

    /// executes convert_bitmap_to_grayscale, meaning it converts Vector of values to grayscale and crops the image
    /// as inputted region area
    #[cfg(not(feature = "lite"))]
    fn grab_screen_image_grayscale(
        &mut self,
        region: &(u32, u32, u32, u32),
    ) -> Result<GrayImage, AutoGuiError> {
        match self {
            Screen::Wayland(s) => s.grab_screen_image_grayscale(region),
            Screen::X11(s) => s.grab_screen_image_grayscale(region),
        }
    }
    #[cfg(not(feature = "lite"))]
    /// captures and saves screenshot of monitors
    fn grab_screenshot(&mut self, image_path: &str) -> Result<(), AutoGuiError> {
        match self {
            Screen::Wayland(s) => s.grab_screenshot(image_path),
            Screen::X11(s) => s.grab_screenshot(image_path),
        }
    }
    #[cfg(not(feature = "lite"))]
    /// first order capture screen function. it captures screen image and stores it as vector in self.pixel_data
    fn capture_screen(&mut self) -> Result<(), AutoGuiError> {
        match self {
            Screen::Wayland(s) => s.capture_screen(),
            Screen::X11(s) => s.capture_screen(),
        }
    }
    #[cfg(not(feature = "lite"))]
    /// convert vector to Luma Imagebuffer
    fn convert_bitmap_to_grayscale(&self) -> Result<GrayImage, AutoGuiError> {
        match self {
            Screen::Wayland(s) => s.convert_bitmap_to_grayscale(),
            Screen::X11(s) => s.convert_bitmap_to_grayscale(),
        }
    }
    #[cfg(not(feature = "lite"))]
    /// convert vector to RGBA ImageBuffer
    fn convert_bitmap_to_rgba(&self) -> Result<RgbaImage, AutoGuiError> {
        match self {
            Screen::Wayland(s) => s.convert_bitmap_to_rgba(),
            Screen::X11(s) => s.convert_bitmap_to_rgba(),
        }
    }

    fn create_keyboard(&mut self) -> crate::core::keyboard::Keyboard {
        match self {
            Screen::Wayland(s) => s.create_keyboard(),
            Screen::X11(s) => s.create_keyboard(),
        }
    }

    fn create_mouse(&mut self) -> crate::core::mouse::Mouse {
        match self {
            Screen::Wayland(s) => s.create_mouse(),
            Screen::X11(s) => s.create_mouse(),
        }
    }
}
