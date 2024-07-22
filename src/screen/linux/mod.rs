extern crate x11;
extern crate image;

use image::{ GrayImage, ImageBuffer, Luma, Rgba};
use std::ptr;
use x11::xlib::*;
use crate::imgtools;

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

impl Screen {
    pub fn new () -> Self {
        unsafe {
            // Open the display (usually ":0")
            let display: *mut _XDisplay = XOpenDisplay(ptr::null());
            if display.is_null() {
                panic!("Unable to open X display");
            }

            // Get the root window
            let screen = XDefaultScreen(display);
            let root = XRootWindow(display, screen);


            // Get the screen dimensions
            let screen_width = XDisplayWidth(display, screen);
            let screen_height = XDisplayHeight(display, screen);
            Screen{
                screen_width: screen_width,
                screen_height: screen_height,
                screen_region_width:0,
                screen_region_height:0,
                pixel_data:vec![0u8; (screen_width * screen_height * 4) as usize],
                display:display,
                root_window:root
            }
        }
    }

    

    pub fn dimension (&self) -> (i32, i32) {
        let dimensions = (self.screen_width, self.screen_height);
        dimensions
    }

    pub fn region_dimension(&self) -> (u32,u32) {
        let dimensions = (self.screen_region_width, self.screen_region_height);
        dimensions
    }

    pub fn destroy(&self) {
        unsafe 
        {
            XCloseDisplay(self.display);
        }  
    }

    pub fn grab_screen_image(&mut self,  region: (u32, u32, u32, u32)) -> ImageBuffer<Rgba<u8>, Vec<u8>>{
        let (x, y, width, height) = region;
        self.screen_region_width = width;
        self.screen_region_height = height;
        self.capture_screen();
        let image = self.convert_bitmap_to_rgba();
        let cropped_image: ImageBuffer<Rgba<u8>, Vec<u8>> = imgtools::cut_screen_region(x, y, width, height, &image);
        cropped_image
    }



    pub fn grab_screen_image_grayscale(&mut self,  region: &(u32, u32, u32, u32)) -> ImageBuffer<Luma<u8>, Vec<u8>>{
        let (x, y, width, height) = region;
        self.screen_region_width = *width;
        self.screen_region_height = *height;
        self.capture_screen();
        let image: ImageBuffer<Luma<u8>, Vec<u8>> = self.convert_bitmap_to_grayscale();
        let cropped_image: ImageBuffer<Luma<u8>, Vec<u8>> = imgtools::cut_screen_region(*x, *y, *width, *height, &image);
        cropped_image
    }

    pub fn grab_screenshot(&mut self, image_path: &str) {
        self.capture_screen();
        let image = self.convert_bitmap_to_rgba(); 
        image.save(image_path).unwrap();
    }


    fn capture_screen(&mut self) {    
        unsafe{
            let ximage = XGetImage(self.display, self.root_window, 0, 0, self.screen_width as u32, self.screen_height as u32, ALLPLANES, ZPixmap);
            if ximage.is_null() {
                panic!("Unable to get X image");
            }

            // Get the image data
            let data = (*ximage).data as *mut u8;
            let data_len = ((*ximage).width * (*ximage).height * ((*ximage).bits_per_pixel / 8)) as usize;
            let slice = std::slice::from_raw_parts(data, data_len);
             // Create an image buffer from the captured data
            let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new((*ximage).width as u32, (*ximage).height as u32);
            let (image_width, image_height) = img.dimensions();
            let mut pixel_data: Vec<u8> = Vec::with_capacity((image_width * image_height * 4) as usize);

            for (x, y, _pixel) in img.enumerate_pixels_mut() {
                let index = ((y * image_width + x) * 4) as usize;
                pixel_data.push(slice[index + 2]); // R
                pixel_data.push(slice[index + 1]); // G
                pixel_data.push(slice[index]);     // B
                pixel_data.push(255);              // A
            }
            self.pixel_data = pixel_data;
            XDestroyImage(ximage);
            
        }
    }

    fn convert_bitmap_to_grayscale(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let mut grayscale_data = Vec::with_capacity((self.screen_width * self.screen_height) as usize);
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
                    grayscale_data
                    ).expect("Couldn't convert to GrayImage")
    }


    fn convert_bitmap_to_rgba(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_raw(
            self.screen_width as u32,
            self.screen_height as u32,
            self.pixel_data.clone(),
        ).expect("Couldn't convert to ImageBuffer")
    }
}

