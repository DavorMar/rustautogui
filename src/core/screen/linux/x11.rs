#[cfg(not(feature = "lite"))]
extern crate image;
extern crate x11;
use crate::core::{
    keyboard::{Keyboard, linux::x11::X11Keyboard},
    mouse::{Mouse, linux::X11Mouse},
};
#[cfg(not(feature = "lite"))]
use crate::errors::ImageProcessingError;
use crate::{core::CaptureableScreen, errors::AutoGuiError, imgtools};
use core::error;
#[cfg(not(feature = "lite"))]
use image::{GrayImage, ImageBuffer, Luma, Rgba, RgbaImage};
#[cfg(not(feature = "lite"))]
use rayon::prelude::*;
use std::ptr;
use x11::xlib::{
    _XDisplay, XCloseDisplay, XDefaultScreen, XDestroyImage, XDisplayHeight, XDisplayWidth,
    XGetImage, XOpenDisplay, XRootWindow, ZPixmap,
};
#[cfg(not(feature = "lite"))]
const ALLPLANES: u64 = 0xFFFFFFFFFFFFFFFF;

#[derive(Debug, Clone)]
pub struct X11Screen {
    pub screen_width: i32,
    pub screen_height: i32,
    pub display: *mut _XDisplay,
    pub root_window: u64,
    #[cfg(not(feature = "lite"))]
    pub pixel_data: Vec<u8>,
}

impl CaptureableScreen for X11Screen {
    fn new() -> Result<Self, AutoGuiError>
    where
        Self: std::marker::Sized,
    {
        unsafe {
            // open the display (usually ":0"). This display pointer will be passed
            // to mouse and keyboard structs aswell
            let display: *mut _XDisplay = XOpenDisplay(ptr::null());
            if display.is_null() {
                panic!(
                    "Error grabbing display. Unable to open X display. Possible x11 issue, check if it is activated and that you're not running wayland"
                );
            }

            // get root window
            let screen = XDefaultScreen(display);
            let root = XRootWindow(display, screen);

            let screen_width = XDisplayWidth(display, screen);
            let screen_height = XDisplayHeight(display, screen);

            Ok(X11Screen {
                screen_width,
                screen_height,
                display,
                root_window: root,
                #[cfg(not(feature = "lite"))]
                pixel_data: vec![0u8; (screen_width * screen_height * 4) as usize],
            })
        }
    }

    /// returns screen dimensions. All monitors included
    fn dimension(&self) -> (i32, i32) {
        (self.screen_width, self.screen_height)
    }

    fn destroy(&mut self) {
        unsafe {
            XCloseDisplay(self.display);
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
        let (x, y, width, height) = region;

        self.capture_screen()?;
        let image = self.convert_bitmap_to_rgba()?;
        let cropped_image: RgbaImage = imgtools::cut_screen_region(x, y, width, height, &image);
        Ok(cropped_image)
    }

    /// executes convert_bitmap_to_grayscale, meaning it converts Vector of values to grayscale and crops the image
    /// as inputted region area
    #[cfg(not(feature = "lite"))]
    fn grab_screen_image_grayscale(
        &mut self,
        region: &(u32, u32, u32, u32),
    ) -> Result<GrayImage, AutoGuiError> {
        let (x, y, width, height) = region;
        self.capture_screen()?;
        let image: GrayImage = self.convert_bitmap_to_grayscale()?;
        let cropped_image: GrayImage = imgtools::cut_screen_region(*x, *y, *width, *height, &image);
        Ok(cropped_image)
    }
    #[cfg(not(feature = "lite"))]
    /// captures and saves screenshot of monitors
    fn grab_screenshot(&mut self, image_path: &str) -> Result<(), AutoGuiError> {
        self.capture_screen()?;
        let image = self.convert_bitmap_to_rgba()?;
        Ok(image.save(image_path)?)
    }
    #[cfg(not(feature = "lite"))]
    /// first order capture screen function. it captures screen image and stores it as vector in self.pixel_data
    fn capture_screen(&mut self) -> Result<(), AutoGuiError> {
        unsafe {
            use image::imageops;

            let ximage = XGetImage(
                self.display,
                self.root_window,
                0,
                0,
                self.screen_width as u32,
                self.screen_height as u32,
                ALLPLANES,
                ZPixmap,
            );
            if ximage.is_null() {
                return Err(AutoGuiError::OSFailure("Error grabbing display image. Unable to get X image. Possible x11 error, check if you're running on x11 and not wayland".to_string()));
            }

            // get the image data
            let data = (*ximage).data as *mut u8;
            let data_len =
                ((*ximage).width * (*ximage).height * ((*ximage).bits_per_pixel / 8)) as usize;
            let slice = std::slice::from_raw_parts(data, data_len);
            // create an image buffer from the captured data
            let mut img = RgbaImage::new((*ximage).width as u32, (*ximage).height as u32);
            let (image_width, image_height) = img.dimensions();

            let mut pixel_data: Vec<u8> =
                Vec::with_capacity((image_width * image_height * 4) as usize);
            for (x, y, _pixel) in img.enumerate_pixels_mut() {
                let index = ((y * image_width + x) * 4) as usize;
                pixel_data.push(slice[index + 2]); // R
                pixel_data.push(slice[index + 1]); // G
                pixel_data.push(slice[index]); // B
                pixel_data.push(255); // A
            }
            self.pixel_data = pixel_data;
            XDestroyImage(ximage);
        }
        Ok(())
    }
    #[cfg(not(feature = "lite"))]
    /// convert vector to Luma Imagebuffer
    fn convert_bitmap_to_grayscale(&self) -> Result<GrayImage, AutoGuiError> {
        let mut grayscale_data =
            Vec::with_capacity((self.screen_width * self.screen_height) as usize);
        for chunk in self.pixel_data.chunks_exact(4) {
            let r = chunk[2] as u32;
            let g = chunk[1] as u32;
            let b = chunk[0] as u32;
            // calculate the grayscale value using the luminance formula
            let gray_value = ((r * 30 + g * 59 + b * 11) / 100) as u8;
            grayscale_data.push(gray_value);
        }
        GrayImage::from_raw(
            self.screen_width as u32,
            self.screen_height as u32,
            grayscale_data,
        )
        .ok_or(ImageProcessingError::new("Failed conversion to grayscale").into())
    }
    #[cfg(not(feature = "lite"))]
    /// convert vector to RGBA ImageBuffer
    fn convert_bitmap_to_rgba(&self) -> Result<RgbaImage, AutoGuiError> {
        ImageBuffer::from_raw(
            self.screen_width as u32,
            self.screen_height as u32,
            self.pixel_data.clone(),
        )
        .ok_or(ImageProcessingError::new("Failed conversion to RGBa").into())
    }

    fn create_keyboard(&mut self) -> Keyboard {
        Keyboard::X11(X11Keyboard::new(self.display))
    }

    fn create_mouse(&mut self) -> Mouse {
        Mouse::X11(X11Mouse::new(self.display, self.root_window))
    }
}
