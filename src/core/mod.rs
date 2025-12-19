pub mod keyboard;
pub mod mouse;
pub mod screen;
#[cfg(not(feature = "lite"))]
pub mod template_match;

#[cfg(not(feature = "lite"))]
use image::{GrayImage, ImageBuffer, Luma, Rgba, RgbaImage};

use crate::core::{
    keyboard::{Keyboard, linux::x11::X11Keyboard},
    mouse::Mouse,
};
#[cfg(not(feature = "lite"))]
use crate::errors::AutoGuiError;

pub trait CaptureableScreen {
    fn new() -> Result<Self, AutoGuiError>
    where
        Self: std::marker::Sized;

    /// returns screen dimensions. All monitors included
    fn dimension(&self) -> (i32, i32);

    fn destroy(&mut self);

    fn create_keyboard(&mut self) -> Keyboard;
    fn create_mouse(&mut self) -> Mouse;

    /// executes convert_bitmap_to_rgba, meaning it converts Vector of values to RGBA and crops the image
    /// as inputted region area. Not used anywhere at the moment
    #[cfg(not(feature = "lite"))]
    fn grab_screen_image(
        &mut self,
        region: (u32, u32, u32, u32),
    ) -> Result<RgbaImage, AutoGuiError>;

    /// executes convert_bitmap_to_grayscale, meaning it converts Vector of values to grayscale and crops the image
    /// as inputted region area
    #[cfg(not(feature = "lite"))]
    fn grab_screen_image_grayscale(
        &mut self,
        region: &(u32, u32, u32, u32),
    ) -> Result<GrayImage, AutoGuiError>;
    #[cfg(not(feature = "lite"))]
    /// captures and saves screenshot of monitors
    fn grab_screenshot(&mut self, image_path: &str) -> Result<(), AutoGuiError>;
    #[cfg(not(feature = "lite"))]
    /// first order capture screen function. it captures screen image and stores it as vector in self.pixel_data
    fn capture_screen(&mut self) -> Result<(), AutoGuiError>;
    #[cfg(not(feature = "lite"))]
    /// convert vector to Luma Imagebuffer
    fn convert_bitmap_to_grayscale(&self) -> Result<GrayImage, AutoGuiError>;
    #[cfg(not(feature = "lite"))]
    /// convert vector to RGBA ImageBuffer
    fn convert_bitmap_to_rgba(&self) -> Result<RgbaImage, AutoGuiError>;
}
