extern crate image;
extern crate x11;

use crate::imgtools;
use core::error;
use image::{GrayImage, ImageBuffer, Luma, Rgba};
use std::ptr;
use x11::xlib::*;

const ALLPLANES: u64 = 0xFFFFFFFFFFFFFFFF;

#[derive(Debug, Clone)]
pub struct Screen {
    pub screen_width: i32,
    pub screen_height: i32,
    pub screen_region_width: u32,
    pub screen_region_height: u32,
    pub pixel_data: Vec<u8>,
    pub display: *mut _XDisplay,
    pub root_window: u64,
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen {
    pub fn new() -> Self {
        unsafe {
            // open the display (usually ":0"). This display pointer will be passed
            // to mouse and keyboard structs aswell
            let display: *mut _XDisplay = XOpenDisplay(ptr::null());
            if display.is_null() {
                panic!("Error grabbing display. Unable to open X display. Possible x11 issue, check if it is activated and that you're not running wayland");
            }

            // get root window
            let screen = XDefaultScreen(display);
            let root = XRootWindow(display, screen);

            let screen_width = XDisplayWidth(display, screen);
            let screen_height = XDisplayHeight(display, screen);
            Screen {
                screen_width,
                screen_height,
                screen_region_width: 0,
                screen_region_height: 0,
                pixel_data: vec![0u8; (screen_width * screen_height * 4) as usize],
                display,
                root_window: root,
            }
        }
    }

    /// returns screen dimensions. All monitors included
    pub fn dimension(&self) -> (i32, i32) {
        (self.screen_width, self.screen_height)
    }

    /// return region dimension which is set up when template is precalculated
    pub fn region_dimension(&self) -> (u32, u32) {
        (self.screen_region_width, self.screen_region_height)
    }

    pub fn destroy(&self) {
        unsafe {
            XCloseDisplay(self.display);
        }
    }

    /// executes convert_bitmap_to_rgba, meaning it converts Vector of values to RGBA and crops the image
    /// as inputted region area. Not used anywhere at the moment
    pub fn grab_screen_image(
        &mut self,
        region: (u32, u32, u32, u32),
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, &'static str> {
        let (x, y, width, height) = region;
        self.screen_region_width = width;
        self.screen_region_height = height;
        self.capture_screen()?;
        let image = self.convert_bitmap_to_rgba()?;
        let cropped_image: ImageBuffer<Rgba<u8>, Vec<u8>> =
            imgtools::cut_screen_region(x, y, width, height, &image);
        Ok(cropped_image)
    }

    /// executes convert_bitmap_to_grayscale, meaning it converts Vector of values to grayscale and crops the image
    /// as inputted region area
    pub fn grab_screen_image_grayscale(
        &mut self,
        region: &(u32, u32, u32, u32),
    ) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, &'static str> {
        let (x, y, width, height) = region;
        self.screen_region_width = *width;
        self.screen_region_height = *height;
        self.capture_screen()?;
        let image: ImageBuffer<Luma<u8>, Vec<u8>> = self.convert_bitmap_to_grayscale()?;
        let cropped_image: ImageBuffer<Luma<u8>, Vec<u8>> =
            imgtools::cut_screen_region(*x, *y, *width, *height, &image);
        Ok(cropped_image)
    }

    /// captures and saves screenshot of monitors
    pub fn grab_screenshot(&mut self, image_path: &str) -> Result<(), String> {
        self.capture_screen()?;
        let image = self.convert_bitmap_to_rgba()?;
        let error_catch = image.save(image_path);
        match error_catch {
            Ok(_) => Ok(()),
            Err(y) => {
                let error_msg = y.to_string();
                Err(error_msg)
            }
        }
    }

    /// first order capture screen function. it captures screen image and stores it as vector in self.pixel_data
    fn capture_screen(&mut self) -> Result<(), &'static str> {
        unsafe {
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
                return Err("Error grabbing display image. Unable to get X image. Possible x11 error, check if you're running on x11 and not wayland. ");
            }

            // get the image data
            let data = (*ximage).data as *mut u8;
            let data_len =
                ((*ximage).width * (*ximage).height * ((*ximage).bits_per_pixel / 8)) as usize;
            let slice = std::slice::from_raw_parts(data, data_len);
            // create an image buffer from the captured data
            let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
                (*ximage).width as u32,
                (*ximage).height as u32,
            );
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

    /// convert vector to Luma Imagebuffer
    fn convert_bitmap_to_grayscale(&self) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, &'static str> {
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
        let grayscale = GrayImage::from_raw(
            self.screen_width as u32,
            self.screen_height as u32,
            grayscale_data,
        );

        grayscale.ok_or("could not convert image to grayscale")
    }

    /// convert vector to RGBA ImageBuffer
    fn convert_bitmap_to_rgba(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, &'static str> {
        ImageBuffer::from_raw(
            self.screen_width as u32,
            self.screen_height as u32,
            self.pixel_data.clone(),
        )
        .ok_or("Failed conversion to RGBa")
    }
}
